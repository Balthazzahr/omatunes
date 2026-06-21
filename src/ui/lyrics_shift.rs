pub fn shift_lrc_timestamps(input: &str, offset_seconds: f64) -> String {
    let mut result = String::new();
    for line in input.lines() {
        let mut line_res = String::new();
        let mut remaining = line;
        
        while let Some(start_idx) = remaining.find('[') {
            // Append everything before the '['
            line_res.push_str(&remaining[..start_idx]);
            let rest = &remaining[start_idx..];
            
            if let Some(end_idx) = rest.find(']') {
                let inside = &rest[1..end_idx];
                if let Some((secs, frac_len_opt)) = parse_lrc_timestamp(inside) {
                    // It is a valid timestamp! Shift it.
                    let shifted = (secs + offset_seconds).max(0.0);
                    
                    // Round to precision to prevent rounding/formatting mismatch (e.g. 59.999 -> 60.00)
                    let precision = frac_len_opt.unwrap_or(0);
                    let factor = 10.0f64.powi(precision as i32);
                    let rounded = (shifted * factor).round() / factor;
                    
                    let mins = (rounded / 60.0).floor() as u32;
                    let secs_rem = rounded - (mins as f64 * 60.0);
                    
                    let formatted = match frac_len_opt {
                        Some(frac_len) => {
                            let width = frac_len + 3; // 2 digits for seconds + 1 dot + frac_len
                            format!(
                                "[{:02}:{:0width$.precision$}]",
                                mins,
                                secs_rem,
                                width = width,
                                precision = frac_len
                            )
                        }
                        None => {
                            format!(
                                "[{:02}:{:02.0}]",
                                mins,
                                secs_rem
                            )
                        }
                    };
                    line_res.push_str(&formatted);
                } else {
                    // Not a valid timestamp, just append the '[' and let it continue
                    line_res.push('[');
                    line_res.push_str(inside);
                    line_res.push(']');
                }
                remaining = &rest[end_idx + 1..];
            } else {
                // No matching ']', so append the rest and finish
                break;
            }
        }
        line_res.push_str(remaining);
        result.push_str(&line_res);
        result.push('\n');
    }
    
    // Preserve trailing newline/non-newline if the input didn't have one
    if !input.is_empty() && !input.ends_with('\n') {
        result.pop();
    }
    result
}

fn parse_lrc_timestamp(s: &str) -> Option<(f64, Option<usize>)> {
    let (min_str, sec_str) = s.split_once(':')?;
    // mins must be digits
    if !min_str.chars().all(|c| c.is_ascii_digit()) || min_str.is_empty() {
        return None;
    }
    
    // sec_str must only contain digits and at most one dot
    let mut dot_count = 0;
    for c in sec_str.chars() {
        if c == '.' {
            dot_count += 1;
        } else if !c.is_ascii_digit() {
            return None;
        }
    }
    if dot_count > 1 || sec_str.is_empty() {
        return None;
    }
    
    let mins = min_str.parse::<f64>().ok()?;
    let secs = sec_str.parse::<f64>().ok()?;
    
    let frac_len = if let Some((_, frac)) = sec_str.split_once('.') {
        Some(frac.len())
    } else {
        None
    };
    
    Some((mins * 60.0 + secs, frac_len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_shift() {
        let input = "[01:12.30] Line 1\n[01:15.50] Line 2";
        let expected = "[01:13.30] Line 1\n[01:16.50] Line 2";
        assert_eq!(shift_lrc_timestamps(input, 1.0), expected);
    }

    #[test]
    fn test_negative_shift_clamping() {
        let input = "[00:00.50] Line 1\n[00:02.00] Line 2";
        let expected = "[00:00.00] Line 1\n[00:01.00] Line 2";
        assert_eq!(shift_lrc_timestamps(input, -1.0), expected);
    }

    #[test]
    fn test_fractional_precision() {
        let input = "[00:10.123] High precision\n[00:10.12] Medium precision";
        let expected = "[00:10.423] High precision\n[00:10.42] Medium precision";
        assert_eq!(shift_lrc_timestamps(input, 0.3), expected);
    }

    #[test]
    fn test_no_timestamp_lines() {
        let input = "Plain lyric line without timestamp\n[00:05.00] With timestamp";
        let expected = "Plain lyric line without timestamp\n[00:06.00] With timestamp";
        assert_eq!(shift_lrc_timestamps(input, 1.0), expected);
    }

    #[test]
    fn test_multiple_timestamps_per_line() {
        let input = "[00:01.00] Word1 [00:02.00] Word2";
        let expected = "[00:02.00] Word1 [00:03.00] Word2";
        assert_eq!(shift_lrc_timestamps(input, 1.0), expected);
    }

    #[test]
    fn test_other_bracketed_tags_preserved() {
        let input = "[ar:Artist Name]\n[al:Album Title]\n[00:10.00] Lyric line";
        let expected = "[ar:Artist Name]\n[al:Album Title]\n[00:11.00] Lyric line";
        assert_eq!(shift_lrc_timestamps(input, 1.0), expected);
    }
}
