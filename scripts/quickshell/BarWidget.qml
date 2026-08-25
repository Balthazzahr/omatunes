import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import Quickshell
import Quickshell.Services.Mpris
import Quickshell.Io
import qs.Commons
import qs.Ui

BarWidget {
    id: root
    moduleName: "omatunes"

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

    // STRICT OmaTUNES matching ONLY - Never fall back to other media players!
    readonly property var activePlayer: {
        var players = Mpris.players ? Mpris.players.values : [];
        for (var i = 0; i < players.length; i++) {
            var p = players[i];
            if (!p) continue;
            var bus = (p.busName || "").toLowerCase();
            var id = (p.identity || "").toLowerCase();
            var entry = (p.desktopEntry || "").toLowerCase();
            if (bus.indexOf("omatunes") !== -1 || id.indexOf("omatunes") !== -1 || entry.indexOf("omatunes") !== -1) {
                return p;
            }
        }
        return null; // Return null if OmaTUNES is not running
    }

    readonly property string playIcon: (activePlayer && activePlayer.playbackState === MprisPlaybackState.Playing) ? "󰏤" : "󰐊"
    readonly property string trackTitle: activePlayer ? (activePlayer.trackTitle || "") : ""
    readonly property string artistName: {
        if (!activePlayer) return "";
        var a = activePlayer.trackArtists;
        if (!a) return "";
        if (Array.isArray(a)) return a.join(", ");
        return String(a);
    }

    // State properties synced via cache file & MPRIS
    property bool isLiked: false
    property bool isShuffleOn: false
    property bool isRepeatOn: false

    Process {
        id: stateCheckProcess
        command: ["bash", "-c", "cat \"$HOME/.cache/omatunes_current_state.json\""]
        stdout: SplitParser {
            onRead: data => {
                try {
                    var obj = JSON.parse(data.trim());
                    if (obj.liked !== undefined) root.isLiked = (obj.liked === true);
                    if (obj.shuffle !== undefined) root.isShuffleOn = (obj.shuffle === true);
                    if (obj.repeat !== undefined) root.isRepeatOn = (obj.repeat === true);
                } catch(e) {}
            }
        }
    }

    Timer {
        interval: 500
        running: true
        repeat: true
        onTriggered: {
            stateCheckProcess.running = true;
            if (activePlayer) {
                if (activePlayer.shuffle !== undefined) root.isShuffleOn = activePlayer.shuffle;
                if (activePlayer.loopStatus !== undefined) {
                    var ls = String(activePlayer.loopStatus);
                    root.isRepeatOn = (ls !== "None" && ls !== "0");
                }
            }
        }
    }

    // Bar label: Music Note glyph ONLY when offline; Title - Artist + Heart when playing
    readonly property string labelText: {
        if (!activePlayer) return ""; // nf-fa-music glyph only when offline
        var titleStr = activePlayer.trackTitle || "";
        var artistStr = artistName;
        var full = (artistStr && titleStr) ? artistStr + " - " + titleStr : (titleStr || artistStr);
        if (!full) return "";
        var displayStr = playIcon + "  " + (full.length > 50 ? full.substring(0, 50) + "…" : full);
        if (root.isLiked) {
            displayStr += "  ";
        }
        return displayStr;
    }

    // Artwork resolution logic
    readonly property string artworkUrl: {
        if (activePlayer && activePlayer.trackArtUrl && activePlayer.trackArtUrl.length > 0) {
            return activePlayer.trackArtUrl;
        }
        if (activePlayer && activePlayer.url && activePlayer.url.length > 0) {
            var u = activePlayer.url;
            var dir = u.substring(0, u.lastIndexOf("/"));
            return dir + "/cover.jpg";
        }
        return "";
    }

    property double trackPosition: activePlayer ? activePlayer.position : 0
    property bool isDraggingSeek: false

    Timer {
        interval: 500
        running: popupCard.open && activePlayer && activePlayer.playbackState === MprisPlaybackState.Playing && !root.isDraggingSeek
        repeat: true
        onTriggered: {
            if (activePlayer) {
                root.trackPosition = activePlayer.position;
            }
        }
    }

    visible: root.activePlayer !== null
    implicitWidth: visible ? button.implicitWidth : 0
    implicitHeight: visible ? button.implicitHeight : 0

    WidgetButton {
        id: button
        anchors.fill: parent
        bar: root.bar
        text: root.labelText
        horizontalMargin: 8
        verticalPadding: 6

        onPressed: function(btn) {
            if (btn === Qt.RightButton) {
                popupCard.open = !popupCard.open;
            } else if (btn === Qt.LeftButton) {
                if (root.activePlayer && root.activePlayer.togglePlaying) {
                    root.activePlayer.togglePlaying();
                } else {
                    root.runCmd("playerctl --player=omatunes play-pause");
                }
            } else if (btn === Qt.MiddleButton) {
                root.runCmd("python3 -c \"import socket; s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM); s.sendto(b'like', ('127.0.0.1', 18888))\"");
            }
        }

        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.NoButton
            onWheel: function(wheel) {
                if (wheel.angleDelta.y > 0) {
                    if (root.activePlayer && root.activePlayer.volume !== undefined) root.activePlayer.volume = Math.min(1.0, root.activePlayer.volume + 0.05);
                    else root.runCmd("playerctl --player=omatunes volume 0.05+");
                } else if (wheel.angleDelta.y < 0) {
                    if (root.activePlayer && root.activePlayer.volume !== undefined) root.activePlayer.volume = Math.max(0.0, root.activePlayer.volume - 0.05);
                    else root.runCmd("playerctl --player=omatunes volume 0.05-");
                }
            }
        }
    }

    PopupCard {
        id: popupCard
        anchorItem: button
        bar: root.bar
        padding: 16
        contentWidth: 380
        contentHeight: 490

        Rectangle {
            anchors.fill: parent
            color: "transparent"

            ColumnLayout {
                anchors.fill: parent
                spacing: 14

                // Album Artwork (75% width of card, 1:1 square) -> Clickable to focus OmaTUNES
                Rectangle {
                    Layout.alignment: Qt.AlignHCenter
                    implicitWidth: parent.width * 0.75
                    implicitHeight: parent.width * 0.75
                    radius: 12
                    color: Color.background
                    border.color: Color.muted
                    border.width: 1
                    clip: true

                    Image {
                        id: albumArt
                        anchors.fill: parent
                        fillMode: Image.PreserveAspectCrop
                        source: root.artworkUrl
                        cache: false
                        asynchronous: true
                        visible: status === Image.Ready
                    }

                    Text {
                        anchors.centerIn: parent
                        visible: albumArt.status !== Image.Ready
                        text: "" // nf-fa-music
                        color: Color.muted
                        font.family: root.bar ? root.bar.fontFamily : "sans-serif"
                        font.pixelSize: 56
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.focusOmatunes()
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
                            text: root.activePlayer ? (root.activePlayer.trackTitle || "") : ""
                            color: Color.foreground
                            font.family: root.bar ? root.bar.fontFamily : "sans-serif"
                            font.pixelSize: Style.font.heading + 2
                            font.bold: true
                            elide: Text.ElideRight
                            Layout.maximumWidth: 260
                            visible: text !== ""

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: root.focusOmatunes()
                            }
                        }

                        // Heart Button: Red (#e67e80 / urgent) when liked, muted when unliked
                        Text {
                            text: "" // nf-fa-heart
                            color: root.isLiked ? "#e67e80" : (likeArea.containsMouse ? Color.accent : Color.muted)
                            font.family: root.bar ? root.bar.fontFamily : "sans-serif"
                            font.pixelSize: 20
                            visible: root.activePlayer !== null

                            MouseArea {
                                id: likeArea
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: {
                                    root.runCmd("python3 -c \"import socket; s=socket.socket(socket.AF_INET,socket.SOCK_DGRAM); s.sendto(b'like',('127.0.0.1',18888))\"");
                                    root.isLiked = !root.isLiked;
                                }
                            }
                        }
                    }

                    Text {
                        Layout.fillWidth: true
                        text: {
                            if (!root.activePlayer) return "";
                            var a = root.artistName;
                            var alb = root.activePlayer.trackAlbum || "";
                            if (a && alb) return a + " — " + alb;
                            if (a) return a;
                            if (alb) return alb;
                            return "";
                        }
                        color: Color.muted
                        font.family: root.bar ? root.bar.fontFamily : "sans-serif"
                        font.pixelSize: Style.font.body
                        elide: Text.ElideRight
                        horizontalAlignment: Text.AlignHCenter
                        visible: text !== ""

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.focusOmatunes()
                        }
                    }
                }



                // Transport Controls: [Shuffle] [Prev] [Play/Pause] [Next] [Repeat]
                RowLayout {
                    Layout.alignment: Qt.AlignHCenter
                    spacing: 20

                    // Shuffle Button (\uf074 - nf-fa-random) with Active Highlight Color
                    Text {
                        text: ""
                        color: root.isShuffleOn ? Color.accent : (shuffleArea.containsMouse ? Color.accent : Color.muted)
                        font.family: root.bar ? root.bar.fontFamily : "sans-serif"
                        font.pixelSize: 20

                        MouseArea {
                            id: shuffleArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                if (root.activePlayer && root.activePlayer.shuffle !== undefined) root.activePlayer.shuffle = !root.isShuffleOn;
                                else root.runCmd("playerctl --player=omatunes shuffle toggle");
                                root.isShuffleOn = !root.isShuffleOn;
                            }
                        }
                    }

                    // Previous Button (\uf048 - nf-fa-backward)
                    Text {
                        text: ""
                        color: prevArea.containsMouse ? Color.accent : Color.foreground
                        font.family: root.bar ? root.bar.fontFamily : "sans-serif"
                        font.pixelSize: 24

                        MouseArea {
                            id: prevArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                if (root.activePlayer) root.activePlayer.previous();
                                else root.runCmd("playerctl --player=omatunes previous");
                            }
                        }
                    }

                    // Play/Pause Button (\uf04b / \uf04c)
                    Rectangle {
                        implicitWidth: 44
                        implicitHeight: 44
                        radius: 22
                        color: playArea.containsMouse ? Color.accent : Color.foreground

                        Text {
                            anchors.centerIn: parent
                            text: (root.activePlayer && root.activePlayer.playbackState === MprisPlaybackState.Playing) ? "" : ""
                            color: Color.background
                            font.family: root.bar ? root.bar.fontFamily : "sans-serif"
                            font.pixelSize: 20
                        }

                        MouseArea {
                            id: playArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                if (root.activePlayer && root.activePlayer.togglePlaying) {
                                    root.activePlayer.togglePlaying();
                                } else {
                                    root.runCmd("playerctl --player=omatunes play-pause");
                                }
                            }
                        }
                    }

                    // Next Button (\uf051 - nf-fa-forward)
                    Text {
                        text: ""
                        color: nextArea.containsMouse ? Color.accent : Color.foreground
                        font.family: root.bar ? root.bar.fontFamily : "sans-serif"
                        font.pixelSize: 24

                        MouseArea {
                            id: nextArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                if (root.activePlayer) root.activePlayer.next();
                                else root.runCmd("playerctl --player=omatunes next");
                            }
                        }
                    }

                    // Repeat Button (\uf021 - nf-fa-repeat) with Active Highlight Color
                    Text {
                        text: ""
                        color: root.isRepeatOn ? Color.accent : (repeatArea.containsMouse ? Color.accent : Color.muted)
                        font.family: root.bar ? root.bar.fontFamily : "sans-serif"
                        font.pixelSize: 20

                        MouseArea {
                            id: repeatArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                root.isRepeatOn = !root.isRepeatOn;
                                root.runCmd("playerctl --player=omatunes loop " + (root.isRepeatOn ? "Playlist" : "None"));
                            }
                        }
                    }
                }

                // Volume Bar (Clicking + Smooth Dragging)
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 10

                    Text {
                        text: "" // nf-fa-volume_up
                        color: Color.muted
                        font.family: root.bar ? root.bar.fontFamily : "sans-serif"
                        font.pixelSize: 16
                    }

                    Rectangle {
                        id: volumeBg
                        Layout.fillWidth: true
                        implicitHeight: 8
                        radius: 4
                        color: Color.background

                        Rectangle {
                            height: parent.height
                            width: {
                                var vol = (root.activePlayer && root.activePlayer.volume !== undefined) ? root.activePlayer.volume : 1.0;
                                return Math.min(parent.width, Math.max(0, parent.width * vol));
                            }
                            radius: 4
                            color: Color.accent
                        }

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor

                            function setVolume(mouseX) {
                                var newVol = Math.min(1.0, Math.max(0.0, mouseX / width));
                                if (root.activePlayer) root.activePlayer.volume = newVol;
                                root.runCmd("playerctl --player=omatunes volume " + newVol.toFixed(2));
                            }

                            onClicked: function(mouse) { setVolume(mouse.x); }
                            onPositionChanged: function(mouse) { if (pressed) setVolume(mouse.x); }
                        }
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
