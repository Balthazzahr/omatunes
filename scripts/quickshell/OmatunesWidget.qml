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
        runCmd("hyprctl dispatch focuswindow class:omatunes || hyprctl dispatch focuswindow title:OmaTUNES || omarchy-launch-or-focus omatunes");
    }

    property var activePlayer: {
        var players = Mpris.players ? Mpris.players.values : [];
        for (var i = 0; i < players.length; i++) {
            var p = players[i];
            if (!p) continue;
            var bus = p.busName || "";
            var id = p.identity || "";
            if (bus.indexOf("omatunes") !== -1 || id.toLowerCase().indexOf("omatunes") !== -1) {
                return p;
            }
        }
        return players.length > 0 ? players[0] : null;
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

    Timer {
        interval: 500
        running: root.popupVisible && activePlayer && activePlayer.playbackState === MprisPlaybackState.Playing
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
                if (!root.activePlayer) return "🎵";
                return root.activePlayer.playbackState === MprisPlaybackState.Playing ? "󰏤" : "󰐊";
            }
            color: Theme.accent
            font.family: Theme.fontFamily
            font.pixelSize: 14
        }

        Text {
            text: {
                if (!root.activePlayer) return "OmaTUNES Offline";
                var artist = root.activePlayer.trackArtists;
                var artistStr = "";
                if (artist) {
                    artistStr = Array.isArray(artist) ? artist.join(", ") : String(artist);
                }
                var title = root.activePlayer.trackTitle || "No Track";
                var fullText = artistStr ? artistStr + " - " + title : title;
                return fullText.length > 35 ? fullText.substring(0, 35) + "…" : fullText;
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
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        cursorShape: Qt.PointingHandCursor

        onClicked: (mouse) => {
            if (mouse.button === Qt.LeftButton) {
                if (root.activePlayer && root.activePlayer.togglePlaying) {
                    root.activePlayer.togglePlaying();
                } else {
                    root.runCmd("playerctl --player=omatunes play-pause");
                }
            } else if (mouse.button === Qt.RightButton) {
                root.popupVisible = !root.popupVisible;
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
