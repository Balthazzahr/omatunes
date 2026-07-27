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

    Rectangle {
        id: card
        implicitWidth: 320
        implicitHeight: 380
        color: Theme.bg
        border.color: Theme.border
        border.width: 1
        radius: 12

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 16
            spacing: 12

            // Top Bar Header
            RowLayout {
                Layout.fillWidth: true
                Text {
                    text: "OmaTUNES Controls"
                    color: Theme.accent
                    font.family: Theme.fontFamily
                    font.pixelSize: 13
                    font.bold: true
                }
                Item { Layout.fillWidth: true }
                Text {
                    text: "󰅖"
                    color: closeArea.containsMouse ? Theme.accent : Theme.muted
                    font.family: Theme.fontFamily
                    font.pixelSize: 14
                    MouseArea {
                        id: closeArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: popup.visible = false
                    }
                }
            }

            // Artwork Container
            Rectangle {
                Layout.alignment: Qt.AlignHCenter
                implicitWidth: 160
                implicitHeight: 160
                radius: 8
                color: Theme.hover
                clip: true

                Image {
                    id: albumArt
                    anchors.fill: parent
                    fillMode: Image.PreserveAspectCrop
                    source: (popup.player && popup.player.artUrl) ? popup.player.artUrl : ""
                    visible: status === Image.Ready
                }

                Text {
                    anchors.centerIn: parent
                    visible: albumArt.status !== Image.Ready
                    text: "󰎈"
                    color: Theme.muted
                    font.family: Theme.fontFamily
                    font.pixelSize: 48
                }
            }

            // Track Info
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2

                Text {
                    Layout.fillWidth: true
                    text: popup.player ? (popup.player.trackTitle || "No Track Playing") : "OmaTUNES Offline"
                    color: Theme.fg
                    font.family: Theme.fontFamily
                    font.pixelSize: 15
                    font.bold: true
                    elide: Text.ElideRight
                    horizontalAlignment: Text.AlignHCenter
                }

                Text {
                    Layout.fillWidth: true
                    text: {
                        if (!popup.player) return "Start OmaTUNES to play music";
                        var artistStr = popup.player.trackArtists ? popup.player.trackArtists.join(", ") : "";
                        var albumStr = popup.player.album || "";
                        if (artistStr && albumStr) return artistStr + " — " + albumStr;
                        if (artistStr) return artistStr;
                        if (albumStr) return albumStr;
                        return "Unknown Artist";
                    }
                    color: Theme.muted
                    font.family: Theme.fontFamily
                    font.pixelSize: 12
                    elide: Text.ElideRight
                    horizontalAlignment: Text.AlignHCenter
                }
            }

            // Progress Slider & Timestamps
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
                            var frac = popup.player.position / popup.player.length;
                            return Math.min(parent.width, Math.max(0, parent.width * frac));
                        }
                        radius: 3
                        color: Theme.accent
                    }

                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: (mouse) => {
                            if (popup.player && popup.player.length > 0) {
                                var targetPos = (mouse.x / width) * popup.player.length;
                                popup.player.setPosition(targetPos);
                            }
                        }
                    }
                }

                RowLayout {
                    Layout.fillWidth: true
                    Text {
                        text: popup.formatTime(popup.player ? popup.player.position : 0)
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

            // Transport Control Buttons
            RowLayout {
                Layout.alignment: Qt.AlignHCenter
                spacing: 16

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
                            if (popup.player) popup.player.playPause();
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
            }

            // Volume Control Bar
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
                        onClicked: (mouse) => {
                            var newVol = mouse.x / width;
                            if (popup.player) popup.player.volume = newVol;
                            popup.runCmd("playerctl --player=omatunes volume " + newVol.toFixed(2));
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
