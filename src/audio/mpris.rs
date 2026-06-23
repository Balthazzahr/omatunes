use mpris_server::{LoopStatus, Metadata, PlaybackStatus, Player, Time};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum MprisCommand {
    Play,
    Pause,
    PlayPause,
    Next,
    Previous,
    Stop,
    SetVolume(f64),
}

#[derive(Debug)]
pub enum MprisUpdate {
    Metadata {
        title: String,
        artist: String,
        album: String,
        duration_us: i64,
        url: String,
    },
    Status(PlaybackStatus),
    Volume(f64),
    Shuffle(bool),
    Loop(LoopStatus),
    Position(std::time::Duration),
}

pub fn launch(
    cmd_tx: mpsc::UnboundedSender<MprisCommand>,
    mut update_rx: mpsc::UnboundedReceiver<MprisUpdate>,
) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        // LocalSet é obrigatório: Player usa Rc internamente (não-Send),
        // e o zbus usa spawn_local para suas tasks D-Bus.
        let local = tokio::task::LocalSet::new();
        local.block_on(&rt, async move {
            let player = match Player::builder("omatunes")
                .identity("Omatunes")
                .can_play(true)
                .can_pause(true)
                .can_go_next(true)
                .can_go_previous(true)
                .can_seek(false)
                .build()
                .await
            {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("MPRIS: falha ao registrar no D-Bus: {e}");
                    return;
                }
            };

            let tx = cmd_tx.clone();
            player.connect_play(move |_| { let _ = tx.send(MprisCommand::Play); });

            let tx = cmd_tx.clone();
            player.connect_pause(move |_| { let _ = tx.send(MprisCommand::Pause); });

            let tx = cmd_tx.clone();
            player.connect_play_pause(move |_| { let _ = tx.send(MprisCommand::PlayPause); });

            let tx = cmd_tx.clone();
            player.connect_next(move |_| { let _ = tx.send(MprisCommand::Next); });

            let tx = cmd_tx.clone();
            player.connect_previous(move |_| { let _ = tx.send(MprisCommand::Previous); });

            let tx = cmd_tx.clone();
            player.connect_stop(move |_| { let _ = tx.send(MprisCommand::Stop); });

            let tx = cmd_tx.clone();
            player.connect_set_volume(move |_, v| { let _ = tx.send(MprisCommand::SetVolume(v)); });

            // Executa o loop D-Bus do player em paralelo com o loop de updates.
            // Sem isso, o player registra no D-Bus mas não processa nenhum comando.
            tokio::task::spawn_local(player.run());

            while let Some(update) = update_rx.recv().await {
                match update {
                    MprisUpdate::Metadata { title, artist, album, duration_us } => {
                        let metadata = Metadata::builder()
                            .title(title)
                            .artist([artist])
                            .album(album)
                            .length(Time::from_micros(duration_us))
                            .build();
                        player.set_metadata(metadata).await.ok();
                    }
                    MprisUpdate::Status(s) => {
                        player.set_playback_status(s).await.ok();
                    }
                    MprisUpdate::Volume(v) => {
                        player.set_volume(v).await.ok();
                    }
                    MprisUpdate::Shuffle(s) => {
                        player.set_shuffle(s).await.ok();
                    }
                    MprisUpdate::Loop(l) => {
                        player.set_loop_status(l).await.ok();
                    }
                    MprisUpdate::Position(d) => {
                        player.set_position(Time::from_micros(d.as_micros() as i64));
                    }
                }
            }
        });
    });
}
