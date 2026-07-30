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
        runCmd("hyprctl dispatch focuswindow class:^omatunes$ || hyprctl dispatch focuswindow title:OmaTUNES || omarchy-launch-or-focus omatunes");
    }

    property bool isLiked: false
    property bool isShuffleOn: false
    property bool isRepeatOn: false

    Process {
        id: stateCheckProcess
        command: ["cat", "$HOME/.cache/omatunes_current_state.json"]
        stdout: SplitParser {
            onRead: data => {
                try {
                    var obj = JSON.parse(data.trim());
                    if (obj.liked !== undefined) popup.isLiked = (obj.liked === true);
                    if (obj.shuffle !== undefined) popup.isShuffleOn = (obj.shuffle === true);
                    if (obj.repeat !== undefined) popup.isRepeatOn = (obj.repeat === true);
                } catch(e) {}
            }
        }
    }

    Timer {
        interval: 500
        running: popup.visible
        repeat: true
        onTriggered: {
            stateCheckProcess.running = true;
            if (player) {
                if (player.shuffle !== undefined) popup.isShuffleOn = player.shuffle;
                if (player.loopStatus !== undefined) {
                    var ls = String(player.loopStatus);
                    popup.isRepeatOn = (ls !== "None" && ls !== "0");
                }
            }
        }
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
    property bool isDraggingSeek: false

    Timer {
        interval: 500
        running: popup.visible && popup.player && popup.player.playbackState === MprisPlaybackState.Playing && !popup.isDraggingSeek
        repeat: true
        onTriggered: {
            if (popup.player) {
                popup.trackPosition = popup.player.position;
            }
        }
    }

    Rectangle {
        id: card
        implicitWidth: 380
        implicitHeight: 490
        color: Theme.bg
        border.color: Theme.border
        border.width: 1
        radius: 12

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 16
            spacing: 14

            // Artwork (75% width of window, 1:1 square) -> Clickable to focus OmaTUNES
            Rectangle {
                Layout.alignment: Qt.AlignHCenter
                implicitWidth: parent.width * 0.75
                implicitHeight: parent.width * 0.75
                radius: 12
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
                    text: "" // nf-fa-music
                    color: Theme.muted
                    font.family: Theme.fontFamily
                    font.pixelSize: 56
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: popup.focusOmatunes()
                }
            }

            // Track Info & Red Heart Button (Clickable -> Focus App)
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 4

                RowLayout {
                    Layout.alignment: Qt.AlignHCenter
                    spacing: 8

                    Text {
                        text: popup.player ? (popup.player.trackTitle || "") : ""
                        color: Theme.fg
                        font.family: Theme.fontFamily
                        font.pixelSize: 17
                        font.bold: true
                        elide: Text.ElideRight
                        Layout.maximumWidth: 260
                        visible: text !== ""

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: popup.focusOmatunes()
                        }
                    }

                    Text {
                        text: "" // nf-fa-heart
                        color: popup.isLiked ? "#e67e80" : (likeArea.containsMouse ? Theme.accent : Theme.muted)
                        font.family: Theme.fontFamily
                        font.pixelSize: 20
                        visible: popup.player !== null

                        MouseArea {
                            id: likeArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                popup.runCmd("echo -n 'like' | nc -u -w0 127.0.0.1 18888 || true");
                                popup.isLiked = !popup.isLiked;
                            }
                        }
                    }
                }

                Text {
                    Layout.fillWidth: true
                    text: {
                        if (!popup.player) return "";
                        var a = popup.artistName;
                        var alb = popup.player.trackAlbum || "";
                        if (a && alb) return a + " — " + alb;
                        if (a) return a;
                        if (alb) return alb;
                        return "";
                    }
                    color: Theme.muted
                    font.family: Theme.fontFamily
                    font.pixelSize: 13
                    elide: Text.ElideRight
                    horizontalAlignment: Text.AlignHCenter
                    visible: text !== ""

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: popup.focusOmatunes()
                    }
                }
            }

            RowLayout {
                Layout.alignment: Qt.AlignHCenter
                spacing: 20

                Text {
                    text: ""
                    color: popup.isShuffleOn ? Theme.accent : (shuffleArea.containsMouse ? Theme.accent : Theme.muted)
                    font.family: Theme.fontFamily
                    font.pixelSize: 20

                    MouseArea {
                        id: shuffleArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            if (popup.player && popup.player.shuffle !== undefined) popup.player.shuffle = !popup.isShuffleOn;
                            else popup.runCmd("playerctl --player=omatunes shuffle toggle");
                            popup.isShuffleOn = !popup.isShuffleOn;
                        }
                    }
                }

                Text {
                    text: ""
                    color: prevArea.containsMouse ? Theme.accent : Theme.fg
                    font.family: Theme.fontFamily
                    font.pixelSize: 24

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
                    implicitWidth: 44
                    implicitHeight: 44
                    radius: 22
                    color: playArea.containsMouse ? Theme.accent : Theme.fg

                    Text {
                        anchors.centerIn: parent
                        text: (popup.player && popup.player.playbackState === MprisPlaybackState.Playing) ? "" : ""
                        color: Theme.bg
                        font.family: Theme.fontFamily
                        font.pixelSize: 20
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
                    text: ""
                    color: nextArea.containsMouse ? Theme.accent : Theme.fg
                    font.family: Theme.fontFamily
                    font.pixelSize: 24

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
                    text: ""
                    color: popup.isRepeatOn ? Theme.accent : (repeatArea.containsMouse ? Theme.accent : Theme.muted)
                    font.family: Theme.fontFamily
                    font.pixelSize: 20

                    MouseArea {
                        id: repeatArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            popup.isRepeatOn = !popup.isRepeatOn;
                            popup.runCmd("playerctl --player=omatunes loop " + (popup.isRepeatOn ? "Playlist" : "None"));
                        }
                    }
                }
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 10

                Text {
                    text: "" // nf-fa-volume_up
                    color: Theme.muted
                    font.family: Theme.fontFamily
                    font.pixelSize: 16
                }

                Rectangle {
                    id: volumeBg
                    Layout.fillWidth: true
                    implicitHeight: 8
                    radius: 4
                    color: Theme.hover

                    Rectangle {
                        height: parent.height
                        width: {
                            var vol = (popup.player && popup.player.volume !== undefined) ? popup.player.volume : 1.0;
                            return Math.min(parent.width, Math.max(0, parent.width * vol));
                        }
                        radius: 4
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
