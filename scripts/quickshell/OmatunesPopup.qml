import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Quickshell
import Quickshell.Services.Mpris
import Quickshell.Io
import "../../theme"

PopupWindow {
    id: popup
    visible: false

    property var player: null

    color: "transparent"

    Process {
        id: execProcess
    }

    function runCmd(cmdStr) {
        execProcess.command = ["bash", "-c", cmdStr];
        execProcess.running = true;
    }

    function focusOmatunes() {
        runCmd("hyprctl dispatch focuswindow class:omatunes || hyprctl dispatch focuswindow title:OmaTUNES || omarchy-launch-or-focus omatunes");
    }

    readonly property string artistName: {
        if (!popup.player) return "";
        var a = popup.player.trackArtists;
        if (!a) return "";
        if (Array.isArray(a)) return a.join(", ");
        return String(a);
    }

    readonly property string artworkUrl: {
        if (popup.player && popup.player.trackArtUrl && popup.player.trackArtUrl.length > 0) {
            return popup.player.trackArtUrl;
        }
        if (popup.player && popup.player.url && popup.player.url.length > 0) {
            var u = popup.player.url;
            var dir = u.substring(0, u.lastIndexOf("/"));
            return dir + "/cover.jpg";
        }
        return "";
    }

    property double trackPosition: popup.player ? popup.player.position : 0

    Timer {
        interval: 500
        running: popup.visible && popup.player && popup.player.playbackState === MprisPlaybackState.Playing
        repeat: true
        onTriggered: {
            if (popup.player) {
                popup.trackPosition = popup.player.position;
            }
        }
    }

    Rectangle {
        id: card
        implicitWidth: 320
        implicitHeight: 360
        color: Theme.bg
        border.color: Theme.border
        border.width: 1
        radius: 12

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 16
            spacing: 12

            // Artwork (Clickable -> Focus App)
            Rectangle {
                Layout.alignment: Qt.AlignHCenter
                implicitWidth: 160
                implicitHeight: 160
                radius: 10
                color: Theme.hover
                clip: true

                Image {
                    id: albumArt
                    anchors.fill: parent
                    fillMode: Image.PreserveAspectCrop
                    source: popup.artworkUrl
                    visible: status === Image.Ready
                }

                Text {
                    anchors.centerIn: parent
                    visible: albumArt.status !== Image.Ready
                    text: "󰎈"
                    color: Theme.muted
                    font.family: Theme.fontFamily
                    font.pixelSize: 54
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: popup.focusOmatunes()
                }
            }

            // Track Info & Like Button (Clickable -> Focus App)
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 4

                RowLayout {
                    Layout.alignment: Qt.AlignHCenter
                    spacing: 8

                    Text {
                        text: popup.player ? (popup.player.trackTitle || "No Track Playing") : "OmaTUNES Offline"
                        color: Theme.fg
                        font.family: Theme.fontFamily
                        font.pixelSize: 15
                        font.bold: true
                        elide: Text.ElideRight
                        Layout.maximumWidth: 240

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: popup.focusOmatunes()
                        }
                    }

                    Text {
                        text: "󰋑"
                        color: likeArea.containsMouse ? Theme.accent : Theme.fg
                        font.family: Theme.fontFamily
                        font.pixelSize: 18

                        MouseArea {
                            id: likeArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: popup.runCmd("/home/user/.local/bin/omatunes_scripts/omatunes_text.py --click like")
                        }
                    }
                }

                Text {
                    Layout.fillWidth: true
                    text: {
                        if (!popup.player) return "Start OmaTUNES to play music";
                        var a = popup.artistName;
                        var alb = popup.player.trackAlbum || "";
                        if (a && alb) return a + " — " + alb;
                        if (a) return a;
                        if (alb) return alb;
                        return "Unknown Artist";
                    }
                    color: Theme.muted
                    font.family: Theme.fontFamily
                    font.pixelSize: 12
                    elide: Text.ElideRight
                    horizontalAlignment: Text.AlignHCenter

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: popup.focusOmatunes()
                    }
                }
            }

            // Progress Slider (Scrubbing Dragging + Live Timer)
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 4

                Rectangle {
                    id: progressBg
                    Layout.fillWidth: true
                    implicitHeight: 6
                    radius: 3
                    color: Theme.hover

                    Rectangle {
                        height: parent.height
                        width: {
                            if (!popup.player || popup.player.length <= 0) return 0;
                            var frac = popup.trackPosition / popup.player.length;
                            return Math.min(parent.width, Math.max(0, parent.width * frac));
                        }
                        radius: 3
                        color: Theme.accent
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor

                        function scrub(mouseX) {
                            if (popup.player && popup.player.length > 0) {
                                var ratio = Math.min(1.0, Math.max(0.0, mouseX / width));
                                var targetSecs = Math.floor(ratio * popup.player.length);
                                popup.trackPosition = targetSecs;
                                popup.runCmd("playerctl --player=omatunes position " + targetSecs);
                            }
                        }

                        onClicked: function(mouse) { scrub(mouse.x); }
                        onPositionChanged: function(mouse) { if (pressed) scrub(mouse.x); }
                    }
                }

                RowLayout {
                    Layout.fillWidth: true
                    Text {
                        text: popup.formatTime(popup.trackPosition)
                        color: Theme.muted
                        font.family: Theme.monoFont
                        font.pixelSize: 11
                    }
                    Item { Layout.fillWidth: true }
                    Text {
                        text: popup.formatTime(popup.player ? popup.player.length : 0)
                        color: Theme.muted
                        font.family: Theme.monoFont
                        font.pixelSize: 11
                    }
                }
            }

            // Transport Controls: [Shuffle] [Prev] [Play/Pause] [Next] [Repeat]
            RowLayout {
                Layout.alignment: Qt.AlignHCenter
                spacing: 14

                Text {
                    text: "󰒹"
                    color: shuffleArea.containsMouse ? Theme.accent : Theme.muted
                    font.family: Theme.fontFamily
                    font.pixelSize: 18

                    MouseArea {
                        id: shuffleArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: popup.runCmd("playerctl --player=omatunes shuffle toggle")
                    }
                }

                Text {
                    text: "󰒮"
                    color: prevArea.containsMouse ? Theme.accent : Theme.fg
                    font.family: Theme.fontFamily
                    font.pixelSize: 22

                    MouseArea {
                        id: prevArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (popup.player) popup.player.previous();
                            else popup.runCmd("playerctl --player=omatunes previous");
                        }
                    }
                }

                Rectangle {
                    implicitWidth: 38
                    implicitHeight: 38
                    radius: 19
                    color: playArea.containsMouse ? Theme.accent : Theme.fg

                    Text {
                        anchors.centerIn: parent
                        text: (popup.player && popup.player.playbackState === MprisPlaybackState.Playing) ? "󰏤" : "󰐊"
                        color: Theme.bg
                        font.family: Theme.fontFamily
                        font.pixelSize: 18
                    }

                    MouseArea {
                        id: playArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (popup.player && popup.player.togglePlaying) popup.player.togglePlaying();
                            else popup.runCmd("playerctl --player=omatunes play-pause");
                        }
                    }
                }

                Text {
                    text: "󰒭"
                    color: nextArea.containsMouse ? Theme.accent : Theme.fg
                    font.family: Theme.fontFamily
                    font.pixelSize: 22

                    MouseArea {
                        id: nextArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (popup.player) popup.player.next();
                            else popup.runCmd("playerctl --player=omatunes next");
                        }
                    }
                }

                Text {
                    text: "󰑖"
                    color: repeatArea.containsMouse ? Theme.accent : Theme.muted
                    font.family: Theme.fontFamily
                    font.pixelSize: 18

                    MouseArea {
                        id: repeatArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: popup.runCmd("playerctl --player=omatunes loop Track || playerctl --player=omatunes loop None")
                    }
                }
            }

            // Volume Control (Scrubbing + Dragging)
            RowLayout {
                Layout.fillWidth: true
                spacing: 8

                Text {
                    text: "󰕾"
                    color: Theme.muted
                    font.family: Theme.fontFamily
                    font.pixelSize: 14
                }

                Rectangle {
                    id: volumeBg
                    Layout.fillWidth: true
                    implicitHeight: 6
                    radius: 3
                    color: Theme.hover

                    Rectangle {
                        height: parent.height
                        width: {
                            var vol = (popup.player && popup.player.volume !== undefined) ? popup.player.volume : 1.0;
                            return Math.min(parent.width, Math.max(0, parent.width * vol));
                        }
                        radius: 3
                        color: Theme.accent
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor

                        function setVolume(mouseX) {
                            var newVol = Math.min(1.0, Math.max(0.0, mouseX / width));
                            if (popup.player) popup.player.volume = newVol;
                            popup.runCmd("playerctl --player=omatunes volume " + newVol.toFixed(2));
                        }

                        onClicked: function(mouse) { setVolume(mouse.x); }
                        onPositionChanged: function(mouse) { if (pressed) setVolume(mouse.x); }
                    }
                }
            }
        }
    }

    function formatTime(seconds) {
        if (!seconds || seconds <= 0 || isNaN(seconds)) return "0:00";
        var mins = Math.floor(seconds / 60);
        var secs = Math.floor(seconds % 60);
        return mins + ":" + (secs < 10 ? "0" : "") + secs;
    }
}
