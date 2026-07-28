import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import Quickshell
import Quickshell.Services.Mpris
import Quickshell.Io
import "../../theme"

Rectangle {
    id: root
    implicitWidth: contentRow.implicitWidth + 16
    implicitHeight: 30
    color: mouseArea.containsMouse ? Theme.hover : "transparent"
    radius: 4

    property bool popupVisible: false

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

    // STRICT OmaTUNES matching ONLY
    property var activePlayer: {
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
        return null;
    }

    property bool isLiked: false
    property bool isShuffleOn: false
    property bool isRepeatOn: false

    Process {
        id: stateCheckProcess
        command: ["cat", "/home/user/.cache/omatunes_current_state.json"]
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
        running: root.popupVisible && activePlayer && activePlayer.playbackState === MprisPlaybackState.Playing && !root.isDraggingSeek
        repeat: true
        onTriggered: {
            if (activePlayer) {
                root.trackPosition = activePlayer.position;
            }
        }
    }

    RowLayout {
        id: contentRow
        anchors.centerIn: parent
        spacing: 6

        Text {
            text: {
                if (!root.activePlayer) return ""; // Music note glyph only when offline
                return root.activePlayer.playbackState === MprisPlaybackState.Playing ? "󰏤" : "󰐊";
            }
            color: Theme.accent
            font.family: Theme.fontFamily
            font.pixelSize: 14
        }

        Text {
            visible: root.activePlayer !== null
            text: {
                if (!root.activePlayer) return "";
                var artist = root.activePlayer.trackArtists;
                var artistStr = "";
                if (artist) {
                    artistStr = Array.isArray(artist) ? artist.join(", ") : String(artist);
                }
                var title = root.activePlayer.trackTitle || "";
                var fullText = (artistStr && title) ? artistStr + " - " + title : (title || artistStr);
                if (!fullText) return "";
                var displayStr = fullText.length > 35 ? fullText.substring(0, 35) + "…" : fullText;
                if (root.isLiked) displayStr += "  ";
                return displayStr;
            }
            color: Theme.fg
            font.family: Theme.fontFamily
            font.pixelSize: 13
            elide: Text.ElideRight
        }
    }

    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.MiddleButton
        cursorShape: Qt.PointingHandCursor

        onClicked: (mouse) => {
            if (mouse.button === Qt.LeftButton) {
                if (root.activePlayer && root.activePlayer.togglePlaying) {
                    root.activePlayer.togglePlaying();
                } else {
                    root.runCmd("/home/user/.local/bin/omatunes_scripts/omatunes_text.py --click play");
                }
            } else if (mouse.button === Qt.RightButton) {
                root.popupVisible = !root.popupVisible;
            } else if (mouse.button === Qt.MiddleButton) {
                root.runCmd("/home/user/.local/bin/omatunes_scripts/omatunes_text.py --click like");
            }
        }

        onWheel: (wheel) => {
            if (wheel.angleDelta.y > 0) {
                root.runCmd("/home/user/.local/bin/omatunes_scripts/omatunes_volume.sh up");
            } else if (wheel.angleDelta.y < 0) {
                root.runCmd("/home/user/.local/bin/omatunes_scripts/omatunes_volume.sh down");
            }
        }
    }

    OmatunesPopup {
        id: popupWindow
        visible: root.popupVisible
        player: root.activePlayer
        anchor.window: barWindow
        anchor.rect.x: root.x
        anchor.rect.y: root.y
        anchor.rect.width: root.width
        anchor.rect.height: root.height
        anchor.edges: Edges.Bottom | Edges.Left
        anchor.gravity: Edges.Bottom | Edges.Right
    }
}
