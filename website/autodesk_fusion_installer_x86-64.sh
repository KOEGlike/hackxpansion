#!/usr/bin/env bash

####################################################################################################
# Name:         Autodesk Fusion 360 - Setup Wizard (Linux)                                         #
# Description:  This file install Autodesk Fusion on your system.                                  #
# Author:       Steve Zabka                                                                        #
# Author URI:   https://cryinkfly.com                                                              #
# License:      MIT                                                                                #
# Copyright (c) 2020-2026                                                                          #
# Time/Date:    11:45/01.06.2026                                                                   #
# Version:      2.1.5-Alpha                                                                        #
####################################################################################################

###############################################################################################################################################################
# THE INITIALIZATION OF DEPENDENCIES STARTS HERE:                                                                                                             #
###############################################################################################################################################################

# CONFIGURATION OF THE COLOR SCHEME:
RED=$'\033[0;31m'
YELLOW=$'\033[0;33m'
GREEN=$'\033[0;32m'
NOCOLOR=$'\033[0m'

# GET THE VALUES OF THE PASSED ARGUMENTS AND ASSIGN THEM TO VARIABLES:
SELECTED_OPTION="$1"
SELECTED_DIRECTORY="$2"
SELECTED_EXTENSIONS="$3"
DOWNLOAD_EXTENSIONS=0
PROTON_VERSION=""
# Detect the Steam installation directory
STEAM_DIRECTORY=""
for STEAM_CANDIDATE in \
    "$HOME/.local/share/Steam" \
    "$HOME/.steam/steam" \
    "$HOME/.steam/root" \
    "$HOME/.steam/debian-installation"; do
    if [ -d "$STEAM_CANDIDATE" ]; then
        STEAM_DIRECTORY="$STEAM_CANDIDATE"
        break
    fi
done
STEAM_COMPAT_DIR="$STEAM_DIRECTORY/compatibilitytools.d"
DESKTOP_DIRECTORY="$HOME/.local/share/applications"
FUSION_DESKTOP_DIRECTORY="$DESKTOP_DIRECTORY/wine/Programs/Autodesk"
WINE_BUILD_DIR="$HOME/fusion-wine-build"

if [ "$SELECTED_DIRECTORY" == "--default" ]; then
    SELECTED_DIRECTORY="$HOME/.autodesk_fusion"
fi

case "$SELECTED_OPTION" in
    --install|--install-fix|--proton=*)
        if [[ "$SELECTED_DIRECTORY" != /*/* ]] || [ "${SELECTED_DIRECTORY%/}" == "${HOME%/}" ]; then
            echo -e "$(gettext "${RED}Invalid installation directory '$SELECTED_DIRECTORY'! Please provide a valid absolute path inside your home directory, e.g. $HOME/.autodesk_fusion${NOCOLOR}")"
            exit 1
        fi
        ;;
esac

# if selected_extensions is set to --full, then all extensions will be installed
if [ "$SELECTED_EXTENSIONS" == "--full" ]; then
    SELECTED_EXTENSIONS="CzechlocalizationforF360,HP3DPrintersforAutodesk®Fusion®,MarkforgedforAutodesk®Fusion®,OctoPrintforAutodesk®Fusion360™,UltimakerDigitalFactoryforAutodeskFusion360™"
    DOWNLOAD_EXTENSIONS=1
fi

REPO_URL="https://codeberg.org/cryinkfly/Autodesk-Fusion-360-on-Linux/raw/branch/main"

# URL to download translations po. files <-- Still in progress!!!
UPDATER_TRANSLATIONS_URL="$REPO_URL/files/setup/locale/update-locale.sh"
declare -A TRANSLATION_URLS=(
    ["cs_CZ"]="$REPO_URL/files/setup/locale/cs_CZ/LC_MESSAGES/autodesk_fusion.po"
    ["de_DE"]="$REPO_URL/files/setup/locale/de_DE/LC_MESSAGES/autodesk_fusion.po"
    ["en_US"]="$REPO_URL/files/setup/locale/en_US/LC_MESSAGES/autodesk_fusion.po"
    ["es_ES"]="$REPO_URL/files/setup/locale/es_ES/LC_MESSAGES/autodesk_fusion.po"
    ["fr_FR"]="$REPO_URL/files/setup/locale/fr_FR/LC_MESSAGES/autodesk_fusion.po"
    ["it_IT"]="$REPO_URL/files/setup/locale/it_IT/LC_MESSAGES/autodesk_fusion.po"
    ["ja_JP"]="$REPO_URL/files/setup/locale/ja_JP/LC_MESSAGES/autodesk_fusion.po"
    ["ko_KR"]="$REPO_URL/files/setup/locale/ko_KR/LC_MESSAGES/autodesk_fusion.po"
    ["pl_PL"]="$REPO_URL/files/setup/locale/pl_PL/LC_MESSAGES/autodesk_fusion.po"
    ["pt_BR"]="$REPO_URL/files/setup/locale/pt_BR/LC_MESSAGES/autodesk_fusion.po"
    ["tr_TR"]="$REPO_URL/files/setup/locale/tr_TR/LC_MESSAGES/autodesk_fusion.po"
    ["zh_CN"]="$REPO_URL/files/setup/locale/zh_CN/LC_MESSAGES/autodesk_fusion.po"
    ["zh_TW"]="$REPO_URL/files/setup/locale/zh_TW/LC_MESSAGES/autodesk_fusion.po"
)

# URL to download winetricks
WINETRICKS_URL="https://raw.githubusercontent.com/Winetricks/winetricks/master/src/winetricks"

# URL to download Fusion360Installer.exe files
AUTODESK_FUSION_INSTALLER_URL="https://dl.appstreaming.autodesk.com/production/installers/Fusion%20Admin%20Install.exe"
#AUTODESK_FUSION_INSTALLER_URL="https://github.com/Lolig4/Autodesk-Fusion-360-for-Linux/releases/download/Fusion_24.03.2026/Fusion_24.03.2026.tar.gz"

# URL to download MicrosoftEdgeWebView2RuntimeInstallerX64.exe
WEBVIEW2_INSTALLER_URL="https://go.microsoft.com/fwlink/?linkid=2124701"
#WEBVIEW2_INSTALLER_URL="https://github.com/aedancullen/webview2-evergreen-standalone-installer-archive/releases/download/109.0.1518.78/MicrosoftEdgeWebView2RuntimeInstallerX64.exe"

# URL to download the patched siappdll.dll file
SIAPPDLL_URL="$REPO_URL/files/extras/patched-dlls/siappdll.dll"

##############################################################################################################################################################################
# CHECK THE REQUIRED PACKAGES FOR THE INSTALLER:                                                                                                                             #
##############################################################################################################################################################################

check_required_packages() {
    # Extracting the Linux distribution ID and version
    DISTRO=$(grep -d skip "^ID=" /etc/*-release | cut -d'=' -f2 | tr -d '"')
    VERSION=$(grep -d skip "^VERSION_ID=" /etc/*-release | cut -d'=' -f2 | tr -d '"')
    DISTRO_VERSION="$DISTRO $VERSION"
    MAJOR=$(echo $VERSION | cut -d'.' -f1)
    MINOR=$(echo $VERSION | cut -d'.' -f2)

    # Example required commands, now including "xrandr" and "bc"
    if [[ $DISTRO_VERSION == *"arch"* ]] || [[ $DISTRO_VERSION == *"manjaro"* ]] || [[ $DISTRO_VERSION == *"endeavouros"* ]] || [[ $DISTRO_VERSION == *"cachyos"* ]]; then
        REQUIRED_COMMANDS=("curl" "lsb_release" "glxinfo" "pkexec" "wget" "awk" "7z" "cabextract" "wbinfo" "systemctl" "bc" "xrandr" "mokutil" "xdg-open" "xdg-mime" "update-desktop-database" "qtpaths")
    else
        REQUIRED_COMMANDS=("curl" "lsb_release" "glxinfo" "pkexec" "wget" "awk" "7z" "cabextract" "wbinfo" "systemctl" "bc" "xrandr" "mokutil" "xdg-open" "xdg-mime" "update-desktop-database")
    fi

    # Additional requirements for building patched Wine/Proton.
    if [[ $SELECTED_OPTION == "--build" ]]; then
        REQUIRED_COMMANDS+=("wine-build-dependency" "ccache" "docker")
    fi

    # Array to store missing commands
    MISSING_COMMANDS=()

    # Check for required commands
    for cmd in "${REQUIRED_COMMANDS[@]}"; do
        echo -e "${YELLOW}Checking for required command: ${cmd} ...${NOCOLOR}"
        if [[ "$cmd" == "wine-build-dependency" ]] || command -v "$cmd" &>/dev/null; then
            case "$cmd" in
                7z)
                    if ! 7z &>/dev/null; then
                        echo -e "${RED}The required command (${cmd}) is not available!${NOCOLOR}"
                        MISSING_COMMANDS+=("$cmd")
                    else
                        echo -e "${GREEN}The required command (${cmd}) is available!${NOCOLOR}"
                    fi
                    ;;
                cabextract)
                    if ! cabextract --version &>/dev/null; then
                        echo -e "${RED}The required command (${cmd}) is not available!${NOCOLOR}"
                        MISSING_COMMANDS+=("$cmd")
                    else
                        echo -e "${GREEN}The required command (${cmd}) is available!${NOCOLOR}"
                    fi
                    ;;
                wbinfo)
                    if ! wbinfo --version &>/dev/null; then
                        echo -e "${RED}The required command (${cmd}) is not available!${NOCOLOR}"
                        MISSING_COMMANDS+=("$cmd")
                    else
                        echo -e "${GREEN}The required command (${cmd}) is available!${NOCOLOR}"
                    fi
                    ;;
                systemctl)
                    if ! systemctl is-active --quiet spacenavd; then
                        echo -e "${YELLOW}The service spacenavd is not active (3D mouse driver - optional, skipping)${NOCOLOR}"
                        # Don't add to MISSING_COMMANDS - spacenavd is optional
                    else
                        echo -e "${GREEN}The service spacenavd is active!${NOCOLOR}"
                    fi
                    ;;
                xrandr)
                    if ! xrandr --version &>/dev/null; then
                        echo -e "${RED}The required command (${cmd}) is not available!${NOCOLOR}"
                        MISSING_COMMANDS+=("$cmd")
                    else
                        echo -e "${GREEN}The required command (${cmd}) is available!${NOCOLOR}"
                    fi
                    ;;
                mokutil)
                    if ! command -v mokutil &>/dev/null; then
                        echo -e "${RED}The required command (${cmd}) is not available!${NOCOLOR}"
                        MISSING_COMMANDS+=("$cmd")
                    else
                        echo -e "${GREEN}The required command (${cmd}) is available!${NOCOLOR}"
                    fi
                    ;;
                xdg-mime)
                    if ! xdg-mime --version &>/dev/null; then
                        echo -e "${RED}The required command (${cmd}) is not available!${NOCOLOR}"
                        MISSING_COMMANDS+=("$cmd")
                    else
                        echo -e "${GREEN}The required command (${cmd}) is available!${NOCOLOR}"
                    fi
                    ;;
                update-desktop-database)
                    if ! update-desktop-database --version &>/dev/null; then
                        echo -e "${RED}The required command (${cmd}) is not available!${NOCOLOR}"
                        MISSING_COMMANDS+=("$cmd")
                    else
                        echo -e "${GREEN}The required command (${cmd}) is available!${NOCOLOR}"
                    fi
                    ;;
                qtpaths)
                    if [[ $DISTRO_VERSION == *"arch"* ]] || [[ $DISTRO_VERSION == *"cachyos"* ]] || [[ $DISTRO_VERSION == *"manjaro"* ]] || [[ $DISTRO_VERSION == *"endeavouros"* ]]; then
                        if ! command -v qtpaths &>/dev/null; then
                            echo -e "${RED}The required command (${cmd}) is not available!${NOCOLOR}"
                            MISSING_COMMANDS+=("$cmd")
                        else
                            echo -e "${GREEN}The required command (${cmd}) is available!${NOCOLOR}"
                        fi
                    fi
                    ;;
                wine-build-dependency)
                    if [[ $DISTRO_VERSION == *"debian"* ]] || [[ $DISTRO == "ubuntu" ]] || [[ $DISTRO_VERSION == *"mint"* ]] || [[ $DISTRO_VERSION == *"pop"* ]] || [[ $DISTRO_VERSION == *"zorin"* ]]; then
                        if ! apt -h 2>/dev/null | grep -q "build-dep"; then
                            echo -e "${RED}The required build dependency tool (apt build-dep) is not available!${NOCOLOR}"
                            MISSING_COMMANDS+=("$cmd")
                        else
                            if apt -s build-dep wine 2>&1 | grep -q '^Inst '; then
                                echo -e "${YELLOW}Wine build dependencies are not fully installed yet.${NOCOLOR}"
                                MISSING_COMMANDS+=("$cmd")
                            elif apt -s build-dep wine &>/dev/null; then
                                echo -e "${GREEN}All Wine build dependencies are already installed!${NOCOLOR}"
                            else
                                echo -e "${RED}Could not verify Wine build dependencies via apt build-dep.${NOCOLOR}"
                                MISSING_COMMANDS+=("$cmd")
                            fi
                        fi
                    elif [[ $DISTRO_VERSION == *"fedora"* ]] || [[ $DISTRO_VERSION == *"nobara"* ]] || [[ $DISTRO_VERSION == *"red"*"hat"*"enterprise"* ]] || [[ $DISTRO_VERSION == *"alma"* ]] || [[ $DISTRO_VERSION == *"rocky"* ]]; then
                        if command -v dnf &>/dev/null; then
                            if ! dnf builddep --help &>/dev/null; then
                                echo -e "${RED}The required build dependency tool (dnf builddep) is not available!${NOCOLOR}"
                                MISSING_COMMANDS+=("$cmd")
                            else
                                if dnf -q builddep --assumeno wine 2>&1 | grep -qi "Nothing to do"; then
                                    echo -e "${GREEN}All Wine build dependencies are already installed!${NOCOLOR}"
                                else
                                    echo -e "${YELLOW}Wine build dependencies are not fully installed yet.${NOCOLOR}"
                                    MISSING_COMMANDS+=("$cmd")
                                fi
                            fi
                        elif ! command -v yum-builddep &>/dev/null; then
                            echo -e "${RED}The required build dependency tool (yum-builddep) is not available!${NOCOLOR}"
                            MISSING_COMMANDS+=("$cmd")
                        else
                            if yum-builddep -q --assumeno wine 2>&1 | grep -qi "Nothing to do"; then
                                echo -e "${GREEN}All Wine build dependencies are already installed!${NOCOLOR}"
                            else
                                echo -e "${YELLOW}Wine build dependencies are not fully installed yet.${NOCOLOR}"
                                MISSING_COMMANDS+=("$cmd")
                            fi
                        fi
                    fi
                    ;;
                ccache)
                    if ! ccache --version &>/dev/null; then
                        echo -e "${RED}The required command (${cmd}) is not available!${NOCOLOR}"
                        MISSING_COMMANDS+=("$cmd")
                    else
                        echo -e "${GREEN}The required command (${cmd}) is available!${NOCOLOR}"
                    fi
                    ;;
                docker)
                    if ! docker --version &>/dev/null; then
                        echo -e "${RED}The required command (${cmd}) is not available! Please install Docker and try again.${NOCOLOR}"
                        exit 1
                    else
                        echo -e "${GREEN}The required command (${cmd}) is available!${NOCOLOR}"
                    fi
                    ;;
                *)
                    echo -e "${GREEN}The required command (${cmd}) is available!${NOCOLOR}"
                    ;;
            esac
        else
            echo -e "${RED}The required command (${cmd}) is not available!${NOCOLOR}"
            MISSING_COMMANDS+=("$cmd")
        fi
    done

    # If there are missing commands, install them
    if [ ${#MISSING_COMMANDS[@]} -gt 0 ]; then
        install_required_packages
    else
        echo -e "${GREEN}All required commands are available!${NOCOLOR}"
    fi

    # Check if Firefox is installed
    firefox_version=$(get_firefox_version)

    # Check if Firefox is installed via Snap and prompt user to install DEB version
    check_install_firefox_deb
}

##############################################################################################################################################################################
# INSTALLATION OF THE REQUIRED PACKAGES FOR THE INSTALLER:                                                                                                                   #
##############################################################################################################################################################################

install_required_packages() {
    echo -e "$(gettext "${YELLOW}The installer will install the required packages for the installation!")${NOCOLOR}"
    echo -e "$(gettext "${RED}Missing package: ${cmd}")${NOCOLOR}"
    sleep 2
    if [[ $DISTRO_VERSION == *"arch"* ]] || [[ $DISTRO_VERSION == *"manjaro"* ]] || [[ $DISTRO_VERSION == *"endeavouros"* ]] || [[ $DISTRO_VERSION == *"cachyos"* ]]; then
        echo -e "$(gettext "${YELLOW}All required packages for the installer will be installed!")${NOCOLOR}"
        sleep 2
        sudo pacman -S gawk cabextract coreutils curl lsb-release mesa-demos mesa-utils p7zip polkit samba wget libspnav xdg-utils bc xorg-xrandr mokutil desktop-file-utils qt5-tools --noconfirm
        sudo systemctl enable --now spacenavd
        echo -e "$(gettext "${GREEN}All required packages for the installer are installed!")${NOCOLOR}"
        sleep 2
    elif [[ $DISTRO_VERSION == *"debian"* ]] || [[ $DISTRO == "ubuntu" ]] \
    || [[ $DISTRO_VERSION == *"mint"* ]] || [[ $DISTRO_VERSION == *"pop"* ]] || [[ $DISTRO_VERSION == *"zorin"* ]]; then
        echo -e "$(gettext "${YELLOW}All required packages for the installer will be installed!")${NOCOLOR}"
        sleep 2
        if [[ $DISTRO == "ubuntu" ]] && { [[ $MAJOR -gt 25 ]] || { [[ $MAJOR -eq 25 ]] && [[ $MINOR -ge 04 ]]; }; }; then
            sudo apt install -y polkitd pkexec gawk cabextract coreutils curl lsb-release mesa-utils p7zip p7zip-full p7zip-rar samba-ad-dc spacenavd winbind wget xdg-utils bc x11-xserver-utils desktop-file-utils
        else
            sudo apt install -y gawk cabextract coreutils curl lsb-release mesa-utils p7zip p7zip-full p7zip-rar policykit-1 samba spacenavd winbind wget xdg-utils bc x11-xserver-utils desktop-file-utils
        fi
        if [[ $SELECTED_OPTION == "--build" ]]; then
            if grep -q "ubuntu.sources" /etc/apt/sources.list.d/ubuntu.sources 2>/dev/null; then
                sudo sed -i 's/^Types: deb$/Types: deb deb-src/' /etc/apt/sources.list.d/ubuntu.sources
            else
                sudo sed -i 's/^# deb-src/deb-src/' /etc/apt/sources.list
            fi
            sudo apt update
            sudo apt install -y ccache
            sudo apt build-dep -y wine
        fi
        sudo systemctl enable --now spacenavd
        echo -e "$(gettext "${GREEN}All required packages for the installer are installed!")${NOCOLOR}"
        sleep 2
    elif [[ $DISTRO_VERSION == *"fedora"* ]] || [[ $DISTRO_VERSION == *"nobara"* ]]; then
        echo -e "$(gettext "${YELLOW}All required packages for the installer will be installed!")${NOCOLOR}"
        sleep 2
        sudo dnf install -y cabextract coreutils curl gawk lsb_release mesa-demos p7zip p7zip-plugins polkit samba-dc samba-winbind samba-winbind-clients spacenavd wget xdg-utils bc xorg-x11-server-utils desktop-file-utils
        if [[ $SELECTED_OPTION == "--build" ]]; then
            sudo dnf install -y ccache dnf-plugins-core
            sudo dnf builddep -y wine --allowerasing
        fi
        sudo systemctl enable --now spacenavd
        echo -e "$(gettext "${GREEN}All required packages for the installer are installed!")${NOCOLOR}"
        sleep 2
    elif [[ $DISTRO_VERSION == *"gentoo"* ]]; then
        echo -e "$(gettext "${YELLOW}All required packages for the installer will be installed!")${NOCOLOR}"
        sleep 2
        sudo emerge -q net-fs/samba app-misc/spacenavd app-arch/cabextract app-arch/p7zip net-misc/curl net-misc/wget sys-apps/coreutils sys-apps/gawk sys-apps/lsb-release sys-auth/polkit x11-apps/mesa-progs x11-misc/xdg-utils sys-apps/bc x11-apps/xrandr dev-util/desktop-file-utils
        # Enable the optional spacenavd service depending on the init system (Gentoo supports both systemd and OpenRC)
        if command -v systemctl &> /dev/null; then
            sudo systemctl enable --now spacenavd
        elif command -v rc-update &> /dev/null; then
            sudo rc-update add spacenavd default
            sudo /etc/init.d/spacenavd start
        fi
        echo -e "$(gettext "${GREEN}All required packages for the installer are installed!")${NOCOLOR}"
        sleep 2
    elif [[ $DISTRO_VERSION == *"nixos"* ]]; then
        echo -e "$(gettext "${YELLOW}All required packages for the installer will be installed!")${NOCOLOR}"
        sleep 2
        sudo nix-env -iA gawk nixos.cabextract nixos.coreutils nixos.curl nixos.lsb_release nixos.mesa-utils nixos.p7zip nixos.polkit nixos.samba nixos.spacenavd nixos.wget nixos.winbind nixos.xdg_utils nixos.bc nixos.xrandr nixos.desktop-file-utils
        sudo systemctl enable --now spacenavd
        echo -e "$(gettext "${GREEN}All required packages for the installer are installed!")${NOCOLOR}"
        sleep 2
    elif [[ $DISTRO_VERSION == *"opensuse"* ]]; then
        echo -e "$(gettext "${YELLOW}All required packages for the installer will be installed!")${NOCOLOR}"
        sleep 2
        sudo zypper install -y cabextract coreutils curl gawk lsb-release Mesa-demo-x p7zip-full polkit samba samba-client samba-winbind spacenavd wget wine xdg-utils bc xorg-x11-server-utils desktop-file-utils
        sudo systemctl enable --now spacenavd
        echo -e "$(gettext "${GREEN}All required packages for the installer are installed!")${NOCOLOR}"
        sleep 2
    elif [[ $DISTRO_VERSION == *"red"*"hat"*"enterprise"* ]] || [[ $DISTRO_VERSION == *"alma"* ]] || [[ $DISTRO_VERSION == *"rocky"* ]]; then
        echo -e "$(gettext "${YELLOW}All required packages for the installer will be installed!")${NOCOLOR}"
        sleep 2
        if command -v dnf &> /dev/null; then # Use dnf for newer distributions
            sudo dnf install -y epel-release
            sudo dnf install -y cabextract coreutils curl gawk lsb_release mesa-demos p7zip p7zip-plugins polkit samba-dc samba-winbind samba-winbind-clients spacenavd wget xdg-utils bc xorg-x11-server-utils desktop-file-utils
            if [[ $SELECTED_OPTION == "--build" ]]; then
                sudo dnf install -y ccache dnf-plugins-core
                sudo dnf builddep -y wine --allowerasing
            fi
        else  # Use yum for older distributions
            sudo yum install -y epel-release 
            sudo yum install -y cabextract coreutils curl gawk lsb_release mesa-demos p7zip p7zip-plugins polkit samba-dc samba-winbind samba-winbind-clients spacenavd wget xdg-utils bc xorg-x11-server-utils desktop-file-utils
            if [[ $SELECTED_OPTION == "--build" ]]; then
                sudo yum install -y ccache yum-utils
                sudo yum-builddep -y wine --allowerasing
            fi
        fi
        sudo systemctl enable --now spacenavd
        echo -e "$(gettext "${GREEN}All required packages for the installer are installed!")${NOCOLOR}"
        sleep 2
    elif [[ $DISTRO_VERSION == *"solus"* ]]; then
        echo -e "$(gettext "${YELLOW}All required packages for the installer will be installed!")${NOCOLOR}"
        sleep 2
        sudo eopkg -y install gawk cabextract coreutils curl lsb-release mesa-utils p7zip p7zip-plugins spacenavd polkit wget winbind xdg-utils bc xrandr desktop-file-utils
        sudo systemctl enable --now spacenavd
        echo -e "$(gettext "${GREEN}All required packages for the installer are installed!")${NOCOLOR}"
        sleep 2
    elif [[ $DISTRO_VERSION == *"void"* ]]; then
        echo -e "$(gettext "${YELLOW}All required packages for the installer will be installed!")${NOCOLOR}"
        sleep 2
        sudo xbps-install -Sy gawk cabextract coreutils curl lsb-release mesa-demos p7zip-full polkit samba-winbind spacenavd wget xdg-utils bc xrandr desktop-file-utils
        sudo ln -s /usr/sbin/spacenavd /etc/sv/spacenavd
        sudo sv enable spacenavd
        sudo sv start spacenavd
        echo -e "$(gettext "${GREEN}All required packages for the installer are installed!")${NOCOLOR}"
        sleep 2
    else
        echo -e "$(gettext "${RED}The installer doesn't support your current Linux distribution $DISTRO_VERSION at this time!")${NOCOLOR}"
        echo -e "$(gettext "${RED}The installer has been terminated!")${NOCOLOR}"
        sleep 2
        exit 1
    fi
}

##############################################################################################################################################################################
# DOWNLOAD THE TRANSLATIONS FOR THE INSTALLER:                                                                                                                              #
##############################################################################################################################################################################

# <-- Still in progress!!!
download_translations() {
    curl -o "./locale/update-locale.sh" "$UPDATER_TRANSLATIONS_URL"
    chmod +x "./locale/update-locale.sh"

    # Curl the translations for the installer
    for locale in "${!TRANSLATION_URLS[@]}"; do
        local TRANSLATION_FILE_URL="${TRANSLATION_URLS[$locale]}"
        local TRANSLATION_FILE_DIRECTORY="./locale/$locale/LC_MESSAGES/autodesk_fusion.po"
        
        mkdir -p "$(dirname "$TRANSLATION_FILE_DIRECTORY")"
        curl -L "$TRANSLATION_FILE_URL" -o "$TRANSLATION_FILE_DIRECTORY"
    done

    source "./locale/update-locale.sh"

    # SET THE TEXTDOMAIN FOR THE INSTALLER:
    TEXTDOMAIN="autodesk_fusion"
    TEXTDOMAINDIR="./locale"

    # Load translations
    export TEXTDOMAIN
    export TEXTDOMAINDIR
}

delete_desktop_files() {
    local REMOVE_LOCATION="$1"
    local FOUND=0
    for DIR in "$FUSION_DESKTOP_DIRECTORY/"*; do
        if [ ! -d "$DIR" ]; then
            continue
        fi
        LOCATION_FILE="$DIR/location.log"
        if [ ! -f "$LOCATION_FILE" ] || [ ! -s "$LOCATION_FILE" ]; then
            echo -e "$(gettext "${RED}location.log file not found or empty in $DIR! Skipping this directory.${NOCOLOR}")"
            continue
        fi
        INSTALL_LOCATION=$(awk 'NR == 1' "$LOCATION_FILE")
        if [ "$INSTALL_LOCATION" == "$REMOVE_LOCATION" ]; then
            rm -rf "$DIR"
            echo -e "$(gettext "${GREEN}Desktop files for the installation at $REMOVE_LOCATION have been deleted!${NOCOLOR}")"
            FOUND=1
        fi
    done
    if (( !FOUND )); then
        echo -e "$(gettext "${RED}No desktop files found for the installation at $REMOVE_LOCATION${NOCOLOR}")"
        exit 1
    fi
}

##############################################################################################################################################################################
# Fix DeviceSettingsProvider.dll path -- Fusion expects it one level above ADPCER/:                                                                                                                                       #
##############################################################################################################################################################################

DeviceSettingsProvider_fix() {
    PRODUCTION_DIR="$WINE_PFX/drive_c/Program Files/Autodesk/webdeploy/production"
    find "$PRODUCTION_DIR" -path "*/ADPCER/DeviceSettingsProvider.dll" | while read -r DLL_PATH; do
        EXPECTED_PATH="$(dirname "$(dirname "$DLL_PATH")")/DeviceSettingsProvider.dll"
            if [[ ! -f "$EXPECTED_PATH" ]]; then
                ln -sf "$DLL_PATH" "$EXPECTED_PATH"
                echo -e "$(gettext "${GREEN}Linked DeviceSettingsProvider.dll: $EXPECTED_PATH${NOCOLOR}")"
            fi
        done
    }

##############################################################################################################################################################################
# CHECK THE OPTIONS FOR THE INSTALLER:                                                                                                                                       #
##############################################################################################################################################################################

check_option() {
    case "$SELECTED_OPTION" in
        --uninstall)
            clear
            echo "$(gettext "${YELLOW}Starting the uninstallation process ...${NOCOLOR}")"

            INSTALLS_LOG="$FUSION_DESKTOP_DIRECTORY/installs.log"

            if [ ! -f "$INSTALLS_LOG" ] || [ ! -s "$INSTALLS_LOG" ]; then
                echo -e "$(gettext "${RED}No installations found! The file $INSTALLS_LOG does not exist or is empty.${NOCOLOR}")"
                exit 1
            fi

            mapfile -t INSTALL_PATHS < <(grep -v '^$' "$INSTALLS_LOG" | sort -u)

            if (( !${#INSTALL_PATHS[@]} )); then
                echo -e "$(gettext "${RED}No installations found in $INSTALLS_LOG!${NOCOLOR}")"
                exit 1
            fi

            # If a path was provided as argument, try to match it directly from installs.log
            if [ -n "$SELECTED_DIRECTORY" ]; then
                MATCH_FOUND=0
                for install_path in "${INSTALL_PATHS[@]}"; do
                    if [ "$install_path" == "$SELECTED_DIRECTORY" ]; then
                        MATCH_FOUND=1
                        break
                    fi
                done
                if (( MATCH_FOUND )); then
                    echo -e "$(gettext "${GREEN}Auto-selected installation: ${YELLOW}$SELECTED_DIRECTORY${NOCOLOR}")"
                else
                    echo -e "$(gettext "${RED}The provided path $SELECTED_DIRECTORY was not found in installs.log!${NOCOLOR}")"
                    SELECTED_DIRECTORY=""
                fi
            fi

            # If no valid path was auto-selected, show the interactive menu
            if [ -z "$SELECTED_DIRECTORY" ]; then
                # List all installations
                echo -e "$(gettext "${GREEN}The following Autodesk Fusion installations were found:${NOCOLOR}")"
                echo ""
                TOTAL_INSTALLS=0
                for install_path in "${INSTALL_PATHS[@]}"; do
                    TOTAL_INSTALLS=$((TOTAL_INSTALLS + 1))
                    if [ -d "$install_path" ]; then
                        echo -e "  ${YELLOW}${TOTAL_INSTALLS}. ${install_path}${NOCOLOR}"
                    else
                        echo -e "  ${YELLOW}${TOTAL_INSTALLS}. ${install_path} ${RED}(directory not found)${NOCOLOR}"
                    fi
                done
                echo ""

                if [ "$TOTAL_INSTALLS" -eq 0 ]; then
                    echo -e "$(gettext "${RED}No valid installations found in $INSTALLS_LOG!${NOCOLOR}")"
                    exit 1
                fi

                echo -ne "$(gettext "Select an installation to uninstall [q to exit]: ")"
                read -n 1 INSTALL_CHOICE
                echo ""

                if [ "$INSTALL_CHOICE" == "q" ] || [ "$INSTALL_CHOICE" == "Q" ]; then
                    echo -e "$(gettext "${YELLOW}The uninstallation process has been canceled!${NOCOLOR}")"
                    exit 0
                fi

                if ! [[ "$INSTALL_CHOICE" =~ ^[0-9]+$ ]] || [ "$INSTALL_CHOICE" -lt 1 ] || [ "$INSTALL_CHOICE" -gt "$TOTAL_INSTALLS" ]; then
                    echo -e "$(gettext "${RED}Invalid selection!${NOCOLOR}")"
                    exit 1
                fi

                SELECTED_DIRECTORY="${INSTALL_PATHS[$((INSTALL_CHOICE - 1))]}"
                echo -e "$(gettext "${GREEN}Selected installation: ${YELLOW}$SELECTED_DIRECTORY${NOCOLOR}")"
            fi

            # Check if the installation directory exists
            if [ ! -d "$SELECTED_DIRECTORY" ]; then
                echo -e "$(gettext "${RED}The installation directory $SELECTED_DIRECTORY does not exist!${NOCOLOR}")"
                awk -v dir="$SELECTED_DIRECTORY" '($0 != dir)' "$INSTALLS_LOG" > "${INSTALLS_LOG}.tmp" && mv "${INSTALLS_LOG}.tmp" "$INSTALLS_LOG"
                echo -e "$(gettext "${GREEN}Entry removed from installs.log!${NOCOLOR}")"
                exit 1
            fi

            read -p "$(gettext "${GREEN}Do you really want to uninstall Autodesk Fusion from $SELECTED_DIRECTORY?${NOCOLOR}") [y/N] " yn
            case $yn in
                [Yy]) echo "$(gettext "${YELLOW}1. Uninstall Autodesk Fusion with all Wineprefixes and components${NOCOLOR}")"
                        echo "$(gettext "${YELLOW}2. Uninstall only a specific Wineprefix of Autodesk Fusion${NOCOLOR}")"
                        read -p "$(gettext "${GREEN}Please select an option: ${NOCOLOR}")" uninstall_option

                        case $uninstall_option in
                            1)  echo -e "$(gettext "${RED}Removing: $SELECTED_DIRECTORY${NOCOLOR}")"
                                rm -rf "$SELECTED_DIRECTORY"
                                delete_desktop_files "$SELECTED_DIRECTORY"
                                # Remove the entry from installs.log
                                awk -v dir="$SELECTED_DIRECTORY" '($0 != dir)' "$INSTALLS_LOG" > "${INSTALLS_LOG}.tmp" && mv "${INSTALLS_LOG}.tmp" "$INSTALLS_LOG"
                                echo "$(gettext "${GREEN}Autodesk Fusion has been uninstalled successfully!${NOCOLOR}")"
                                exit 0;;
                            2)  if [ ! -d "$SELECTED_DIRECTORY/wineprefixes/" ]; then
                                    echo -e "$(gettext "${RED}No wineprefixes directory found in $SELECTED_DIRECTORY!${NOCOLOR}")"
                                    exit 1
                                fi
                                echo "$(gettext "${GREEN}Listing all Wineprefixes of Autodesk Fusion in the ${SELECTED_DIRECTORY}/wineprefixes/ directory${NOCOLOR}")"
                                # Initialize counter
                                COUNTER=1
                                for wp in "$SELECTED_DIRECTORY/wineprefixes/"*; do
                                    [ -d "$wp" ] || continue
                                    # Display the counter and wineprefix name
                                    echo "$(gettext "${YELLOW}${COUNTER}. $(basename "$wp")${NOCOLOR}")"
                                    # Increment the counter
                                    COUNTER=$((COUNTER + 1))
                                done
                                if [ "$COUNTER" -eq 1 ]; then
                                    echo -e "$(gettext "${RED}No wineprefixes found!${NOCOLOR}")"
                                    exit 1
                                fi
                                read -p "$(gettext "${RED}Enter the number of the Wineprefix you want to uninstall or type 'exit' to cancel the process: ${NOCOLOR}")" DEL_SELECTED_WINEPREFIX
                                case $DEL_SELECTED_WINEPREFIX in
                                    exit) echo "$(gettext "${GREEN}The uninstallation process has been canceled!${NOCOLOR}")"
                                        exit 0;;
                                    *) DEL_SELECTED_WINEPREFIX=$(ls "$SELECTED_DIRECTORY/wineprefixes/" | sed -n "${DEL_SELECTED_WINEPREFIX}p")
                                        if [ -z "$DEL_SELECTED_WINEPREFIX" ]; then
                                            echo -e "$(gettext "${RED}Invalid selection!${NOCOLOR}")"
                                            exit 1
                                        fi
                                        echo -e "$(gettext "${YELLOW}Removing Wineprefix: $SELECTED_DIRECTORY/wineprefixes/$DEL_SELECTED_WINEPREFIX${NOCOLOR}")"
                                        rm -rf "$SELECTED_DIRECTORY/wineprefixes/$DEL_SELECTED_WINEPREFIX"
                                        echo "$(gettext "${GREEN}The selected Wineprefix has been uninstalled successfully!${NOCOLOR}")"
                                        exit 0;;
                                esac;;  
                            *) echo "$(gettext "${RED}Please select a valid option!${NOCOLOR}")"
                                exit 1;;
                        esac;;  
                *) echo -e "$(gettext "${GREEN}The uninstallation process has been canceled!")${NOCOLOR}"
                    exit 0;;
            esac;;

        --install|--install-fix|--proton=*)
            echo -e "$(gettext "${GREEN}Starting the installation process ...${NOCOLOR}")"
            sleep 1
            echo -e "$(gettext "${GREEN}Linux distribution: ${YELLOW}$DISTRO_VERSION${NOCOLOR}")"
            sleep 1
            if [[ "$SELECTED_OPTION" == --proton=* ]]; then
                PROTON_VERSION="${1#--proton=}"
                SELECTED_OPTION="--proton"
            fi
            echo -e "$(gettext "${GREEN}Selected option: ${YELLOW}$SELECTED_OPTION${NOCOLOR}")"
            sleep 1
            if [[ -n "$PROTON_VERSION" && "$SELECTED_OPTION" == "--proton" ]]; then
                echo -e "$(gettext "${GREEN}Selected Proton version: ${YELLOW}$PROTON_VERSION${NOCOLOR}")"
                PROTON_DIRECTORY="$STEAM_COMPAT_DIR/$PROTON_VERSION"
                PROTONPREFIX_DIRECTORY="$SELECTED_DIRECTORY/protonprefix"
                WINE_PFX="$PROTONPREFIX_DIRECTORY/pfx"
                sleep 1
            elif [[ "$SELECTED_OPTION" == "--install" || "$SELECTED_OPTION" == "--install-fix" ]]; then
                WINE_PFX="$SELECTED_DIRECTORY/wineprefixes/default"
            else
                echo -e "$(gettext "${RED}Invalid option! Please use the --install, --proton or --proton=<version> flag!")${NOCOLOR}"
                exit 1
            fi
            echo -e "$(gettext "${GREEN}Selected directory: ${YELLOW}$SELECTED_DIRECTORY${NOCOLOR}")"
            sleep 1
            echo -e "$(gettext "${GREEN}Selected extensions: ${YELLOW}$SELECTED_EXTENSIONS${NOCOLOR}")"
            sleep 1
            deactivate_window_not_responding_dialog
            create_data_structure
            check_secure_boot
            check_ram
            check_gpu_driver
            check_gpu_vram
            check_disk_space
            download_files
            if [[ "$SELECTED_OPTION" == "--proton" ]]; then
                check_steam_proton
            fi
            check_and_install_wine
            wine_autodesk_fusion_install
            DeviceSettingsProvider_fix
            autodesk_fusion_patch_siappdll
            wine_autodesk_fusion_install_extensions
            autodesk_fusion_shortcuts_load
            autodesk_fusion_safe_logfile
            reset_window_not_responding_dialog
            xdg-open "https://cryinkfly.com/contributors/"
            run_wine_autodesk_fusion
            exit 0;;
        --build)
            check_and_install_wine
            case "$SELECTED_DIRECTORY" in
                wine-fix)
                build_patched_wine
                exit 0;;
                proton-fix)
                build_patched_proton
                exit 0;;
                *)
                echo -e "$(gettext "${RED}Invalid build option! Please use --build wine to build the patched Wine version!${NOCOLOR}")"
                exit 1;;
            esac;;
        *)
            echo -e "$(gettext "${RED}Invalid option! Please use the --install, --proton, --proton=<version> or --uninstall flag!")${NOCOLOR}"
            exit 1;;
    esac
}

##############################################################################################################################################################################
# DEACTIVATE THE WINDOW NOT RESPONDING DIALOG:                                                                                                                               #
##############################################################################################################################################################################

deactivate_window_not_responding_dialog() {
    # Check if desktop environment is GNOME
    if [ "$XDG_CURRENT_DESKTOP" = "GNOME" ]; then
        # Disable the "Window not responding" Dialog in GNOME for 30 minutes:
        echo -e "$(gettext "${YELLOW}The 'Window not responding' Dialog in GNOME will be disabled for 30 minutes!")${NOCOLOR}"
        gsettings set org.gnome.mutter check-alive-timeout 1800000
    fi
}

##############################################################################################################################################################################
# CREATE THE DATA STRUCTURE FOR THE INSTALLER:                                                                                                                               #
##############################################################################################################################################################################

create_data_structure() {
    rm -rf "$WINE_PFX"
    mkdir -p "$SELECTED_DIRECTORY/bin" \
        "$SELECTED_DIRECTORY/downloads/extensions" \
        "$SELECTED_DIRECTORY/logs" \
        "$SELECTED_DIRECTORY/.desktop" \
        "$SELECTED_DIRECTORY/resources/graphics" \
        "$SELECTED_DIRECTORY/resources/styles" \
        "$WINE_PFX"
}

##############################################################################################################################################################################
# CHECK IF SECURE BOOT IS DEACTIVATED ON A LINUX SYSTEM FOR LOADING DRIVER MODULES (FOR EXAMPLE: NVIDIA GPU DRIVER):                                                         #
##############################################################################################################################################################################

# Function to check if Secure Boot is activated
check_secure_boot() {
    if ! command -v mokutil &> /dev/null; then
        echo "${RED} mokutil command not found. Please install it to check Secure Boot status.${NOCOLOR}"
        exit 1
    fi

    # Check if Secure Boot is enabled
    if mokutil --sb-state | grep -qE 'Secure Boot enabled|SecureBoot enabled'; then
        echo "Secure Boot is enabled."
        SECURE_BOOT=1
    else
        echo "Secure Boot is not enabled."
        SECURE_BOOT=0
    fi
}

##############################################################################################################################################################################
# CHECKING THE MINIMUM RAM (RANDOM ACCESS MEMORY) REQUIREMENT:                                                                                                               #
##############################################################################################################################################################################

check_ram() {
    # Get total RAM space in kilobytes
    GET_RAM_KILOBYTES=$(grep MemTotal /proc/meminfo | awk '{print $2}')
    
    # Check if the total memory is greater than 4000 Megabytes
    if awk "BEGIN {exit !($GET_RAM_KILOBYTES > 4000 * 1024)}"; then
        CONVERT_RAM_GIGABYTES=$(awk "BEGIN {printf \"%.2f\", $GET_RAM_KILOBYTES / 1024 / 1024}")
        echo -e "$(gettext "${GREEN}The total RAM (Random Access Memory) is greater than 4 GByte ($CONVERT_RAM_GIGABYTES GByte) and Autodesk Fusion will run more stable later!${NOCOLOR}")"
    else
        CONVERT_RAM_GIGABYTES=$(awk "BEGIN {printf \"%.2f\", $GET_RAM_KILOBYTES / 1024 / 1024}")
        echo -e "$(gettext "${RED}The total RAM (Random Access Memory) is not greater than 4 GByte ($CONVERT_RAM_GIGABYTES GByte) and Autodesk Fusion may run unstable later with insufficient RAM memory!${NOCOLOR}")"
        read -p "$(gettext "${YELLOW}Are you sure you want to continue with the installation? [y/N]${NOCOLOR}")" INSTALL_CONFIRM_CHOICE
        case "$INSTALL_CONFIRM_CHOICE" in 
            [Yy]) 
                echo -e "$(gettext "${YELLOW}Continuing with the installation...${NOCOLOR}")"
                ;;
            *) 
                echo -e "$(gettext "${RED}The installer has been terminated!${NOCOLOR}")"
                exit 0;;
        esac
    fi
}

##############################################################################################################################################################################
# CHECK GPU DRIVER FOR THE INSTALLER:                                                                                                                                        #
##############################################################################################################################################################################

check_gpu_driver() {
    echo -e "$(gettext "${YELLOW}Checking the GPU vendor for the installer...${NOCOLOR}")"

    # Initialize flags
    NVIDIA_PRESENT=0
    AMD_PRESENT=0
    INTEL_PRESENT=0

    # Detect GPU vendor using lspci first (works headless/without display server),
    # fall back to glxinfo if lspci is unavailable.
    if command -v lspci >/dev/null 2>&1; then
        GPU_VENDOR=$(lspci | grep -E "VGA|3D|Display" | grep -oE "NVIDIA|AMD|Intel" | head -n1)
    elif command -v glxinfo >/dev/null 2>&1; then
        GPU_VENDOR=$(glxinfo -B 2>/dev/null | grep "OpenGL vendor" | grep -oiE "NVIDIA|AMD|Intel" | head -n1)
    fi

    if [[ "$GPU_VENDOR" == "AMD" ]]; then
        AMD_PRESENT=1
    elif [[ "$GPU_VENDOR" == "NVIDIA" ]]; then
        NVIDIA_PRESENT=1
    elif [[ "$GPU_VENDOR" == "INTEL" ]]; then
        INTEL_PRESENT=1
    elif [[ -z "$GPU_VENDOR" ]]; then
        echo -e "$(gettext "${YELLOW}WARNING: Could not detect GPU vendor automatically. Defaulting to OpenGL.${NOCOLOR}")"
        GPU_VENDOR="Unknown"
    fi

    echo -e "$(gettext "${GREEN}Detected GPU vendor: $GPU_VENDOR${NOCOLOR}")"

    echo -e "$(gettext "${YELLOW}Checking the GPU drivers for the installer...${NOCOLOR}")"

    if (( !SECURE_BOOT )); then
        # If Secure Boot is disabled, check NVIDIA GPU
        if nvidia-smi &>/dev/null; then
            NVIDIA_PRESENT=1
            NVIDIA_VRAM=$(nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits | head -n1)
            echo -e "$(gettext "${GREEN}NVIDIA GPU detected with ${NVIDIA_VRAM}MB VRAM${NOCOLOR}")"
        fi
    fi

    # Only probe Intel/AMD if NVIDIA wasn't found (avoids running glxinfo unnecessarily)
    if  (( !NVIDIA_PRESENT )) ; then
        if command -v glxinfo >/dev/null 2>&1; then
            INTEL_AMD_GPU=$(glxinfo 2>/dev/null | grep "OpenGL vendor string" | cut -d: -f2 | tr -d ' ')
            INTEL_AMD_VRAM=$(glxinfo 2>/dev/null | grep -i "Video memory" | grep -Eo '[0-9]+MB' | grep -Eo '[0-9]+' | head -n1)
        else
            # glxinfo not available, fall back to vendor string from lspci detection above
            INTEL_AMD_GPU="$GPU_VENDOR"
            INTEL_AMD_VRAM=""
        fi

        if [[ $INTEL_AMD_GPU == "AMD" ]]; then
            AMD_PRESENT=1
            AMD_VRAM="${INTEL_AMD_VRAM:-0}"
            echo -e "$(gettext "${GREEN}${INTEL_AMD_GPU} GPU recognized with ${AMD_VRAM}MB VRAM${NOCOLOR}")"
        elif [[ $INTEL_AMD_GPU == "Intel" ]]; then
            INTEL_PRESENT=1
            INTEL_VRAM="${INTEL_AMD_VRAM:-0}"
            echo -e "$(gettext "${GREEN}${INTEL_AMD_GPU} GPU recognized with ${INTEL_VRAM}MB VRAM${NOCOLOR}")"
        fi
    fi

    OLDER_NVIDIA_GPU=0
    if lspci | grep -q "GTX 970"; then
        OLDER_NVIDIA_GPU=1
    fi

    if (( (SECURE_BOOT && NVIDIA_PRESENT) || OLDER_NVIDIA_GPU )); then
        # If Secure Boot is enabled and the NVIDIA GPU is detected, the NVIDIA GPU should use OpenGL.
        GPU_DRIVER="OpenGL"
        GET_VRAM_MEGABYTES="$NVIDIA_VRAM"
        echo -e "$(gettext "${GREEN}Secure Boot is enabled. The OpenGL GPU driver is being used for the NVIDIA GPU.${NOCOLOR}")"
    else 
        echo -e "$(gettext "${GREEN}Secure Boot is disabled. Checking available GPU drivers...${NOCOLOR}")"
        # If Secure Boot is disabled, handle GPU selection
        if (( NVIDIA_PRESENT && (INTEL_PRESENT || AMD_PRESENT) )); then
            echo -e "$(gettext "${YELLOW}Multiple GPUs detected. Please select which one to use (default is DXVK):${NOCOLOR}")"
            echo "1) NVIDIA"
            echo "2) ${INTEL_AMD_GPU}"
            read -p "Enter your choice (1 or 2): " gpu_choice
            
            case $gpu_choice in
                1)
                    GPU_DRIVER="DXVK"
                    GET_VRAM_MEGABYTES="$NVIDIA_VRAM"
                    echo -e "$(gettext "${GREEN}NVIDIA GPU selected. The DXVK GPU driver will be used for installation.${NOCOLOR}")"
                    ;;
                2)
                    GPU_DRIVER="OpenGL"
                    GET_VRAM_MEGABYTES="$INTEL_AMD_VRAM"
                    echo -e "$(gettext "${GREEN}The OpenGL GPU fallback driver is used for the installation.${NOCOLOR}")"
                    ;;
                *)
                    GPU_DRIVER="OpenGL"
                    GET_VRAM_MEGABYTES="$INTEL_VRAM"
                    ;;
            esac
        elif (( NVIDIA_PRESENT )); then
            GPU_DRIVER="DXVK"
            GET_VRAM_MEGABYTES="$NVIDIA_VRAM"
            echo -e "$(gettext "${GREEN}The DXVK GPU driver is used for the installation.${NOCOLOR}")"
        elif (( AMD_PRESENT )); then
            # Detect if AMD GPU is an APU (integrated) or discrete
            AMD_IS_APU=0
            if command -v lspci >/dev/null 2>&1; then
                AMD_PCI_INFO=$(lspci | grep -E "VGA|3D|Display" | grep -iE "AMD|ATI|Radeon")
                # Match known APU codenames
                if echo "$AMD_PCI_INFO" | grep -qiE \
                    "renoir|raven|picasso|cezanne|rembrandt|mendocino|phoenix|hawk|barcelo|lucienne|vangogh"; then
                    AMD_IS_APU=1
                fi
                # Secondary check — no PCIe link means integrated
                AMD_PCI_SLOT=$(lspci | grep -E "VGA|3D|Display" | grep -iE "AMD|ATI|Radeon" | awk '{print $1}')
                if [[ -n "$AMD_PCI_SLOT" ]]; then
                    AMD_LINK=$(lspci -vv -s "$AMD_PCI_SLOT" 2>/dev/null | grep -i "LnkCap" | head -1)
                    if [[ -z "$AMD_LINK" ]]; then
                        AMD_IS_APU=1
                    fi
                fi
            fi
            if (( AMD_IS_APU )); then
                GPU_DRIVER="DXVK"
                GET_VRAM_MEGABYTES="$AMD_VRAM"
                echo -e "$(gettext "${GREEN}AMD APU detected. Using $GPU_DRIVER driver for compatibility.${NOCOLOR}")"
            else
                GPU_DRIVER="DXVK"
                GET_VRAM_MEGABYTES="$AMD_VRAM"
                echo -e "$(gettext "${GREEN}AMD discrete GPU detected. Using $GPU_DRIVER  driver.${NOCOLOR}")"
            fi
        elif (( INTEL_PRESENT )); then
            GPU_DRIVER="OpenGL"
            GET_VRAM_MEGABYTES="$INTEL_VRAM"
            echo -e "$(gettext "${GREEN}The OpenGL GPU fallback driver is used for the installation.${NOCOLOR}")"
        else
            echo -e "$(gettext "${RED}No GPU driver detected on your system!${NOCOLOR}")"
            GET_VRAM_MEGABYTES=0
        fi
    fi

    sleep 2

    # Get the current display resolution of the main monitor if more than one is connected.
    MONITOR_RESOLUTION=$(xrandr 2>/dev/null | grep 'primary' | awk '{print $4}' | cut -d'+' -f1)

    # If the $MONITOR_RESOLUTION value is empty, set it to "1920x1080"
    if [ -z "$MONITOR_RESOLUTION" ]; then
        MONITOR_RESOLUTION="1920x1080"
    fi

    # Output the resolution
    echo -e "$(gettext "${GREEN}Main monitor resolution: $MONITOR_RESOLUTION ${NOCOLOR}")"

    sleep 2
}

##############################################################################################################################################################################
# CHECKING THE MINIMUM VRAM (VIDEO RAM) REQUIREMENT:                                                                                                                         #
##############################################################################################################################################################################

check_gpu_vram() {
    # Get the total memory of the graphics card in megabytes from check_gpu_driver

    if [ -z "$GET_VRAM_MEGABYTES" ]; then
        echo -e "$(gettext "${RED}Could not determine VRAM size.${NOCOLOR}")"
        exit 1
    fi
    
    # Check if the total memory is greater than 1000 Megabytes
    if awk -v vram="$GET_VRAM_MEGABYTES" 'BEGIN {exit !(vram > 1000)}'; then
        CONVERT_RAM_GIGABYTES=$(awk "BEGIN {printf \"%.2f\", $GET_VRAM_MEGABYTES / 1000}")
        echo -e "$(gettext "${GREEN}The total VRAM (Video RAM) is greater than 1 GByte (${CONVERT_RAM_GIGABYTES} GByte) and Autodesk Fusion will run more stable later!${NOCOLOR}")"
    else
        CONVERT_RAM_GIGABYTES=$(awk "BEGIN {printf \"%.2f\", $GET_VRAM_MEGABYTES / 1000}")
        echo -e "$(gettext "${RED}The total VRAM (Video RAM) is not greater than 1 GByte (${CONVERT_RAM_GIGABYTES} GByte) and Autodesk Fusion may run unstable later with insufficient VRAM memory!${NOCOLOR}")"
        read -p "$(gettext "${YELLOW}Are you sure you want to continue with the installation? [y/N]${NOCOLOR}")" VRAM_CONFIRM_CHOICE
        case "$VRAM_CONFIRM_CHOICE" in
            [Yy]) echo -e "$(gettext "${GREEN}Continuing with the installation...${NOCOLOR}")"
                ;;
            *) echo -e "$(gettext "${RED}The installer has been terminated!${NOCOLOR}")"
                exit 0;;
        esac
    fi
}

##############################################################################################################################################################################
# CHECKING THE MINIMUM DISK SPACE (DEFAULT: HOME-PARTITION) REQUIREMENT:                                                                                                     #
##############################################################################################################################################################################

check_disk_space() {
    # Get the free disk space in the selected directory (or its closest existing parent)
    DISK_CHECK_DIR="$SELECTED_DIRECTORY"
    while [[ ! -d "$DISK_CHECK_DIR" ]] && [[ "$DISK_CHECK_DIR" != "/" ]]; do
        DISK_CHECK_DIR="$(dirname "$DISK_CHECK_DIR")"
    done
    GET_DISK_SPACE=$(df -h "$DISK_CHECK_DIR" 2>/dev/null | awk 'NR==2 {print $4}')

    if [[ -z "$GET_DISK_SPACE" ]]; then
        echo -e "${RED}Failed to retrieve disk space information. Ensure the directory exists and try again.${NOCOLOR}"
        exit 1
    fi

    echo -e "$(gettext "${GREEN}The free disk memory size is: $GET_DISK_SPACE${NOCOLOR}")"

    # Extract numerical value and unit, and replace comma with dot
    DISK_SPACE_NUM=$(echo "$GET_DISK_SPACE" | sed 's/[A-Za-z]//g' | sed 's/,/./g')
    DISK_SPACE_UNIT=$(echo "$GET_DISK_SPACE" | sed 's/[0-9.,]//g')

    # Convert to gigabytes
    case $DISK_SPACE_UNIT in
        G) DISK_SPACE_GB=$DISK_SPACE_NUM ;;
        M) DISK_SPACE_GB=$(echo "scale=2; $DISK_SPACE_NUM / 1024" | bc) ;;
        T) DISK_SPACE_GB=$(echo "scale=2; $DISK_SPACE_NUM * 1024" | bc) ;;
        *) DISK_SPACE_GB=0 ;;
    esac

    # Check if the free disk space is greater than 10GB
    if (( $(echo "$DISK_SPACE_GB > 10" | bc -l) )); then
        echo -e "$(gettext "${GREEN}The free disk memory size is greater than 10GB.${NOCOLOR}")"
    else
        echo -e "$(gettext "${YELLOW}There is not enough disk free memory to continue installing Fusion on your system!${NOCOLOR}")"
        echo -e "$(gettext "${YELLOW}Make more space in your selected disk or select a different hard drive.${NOCOLOR}")"
        echo -e "$(gettext "${RED}The installer has been terminated!${NOCOLOR}")"
        exit 1
    fi
}

function check_steam_proton() {
    # Check if Proton is installed and use Proton to run Autodesk Fusion 360
    if [ -d "$STEAM_DIRECTORY" ]; then
        echo -e "$(gettext "${GREEN}Steam is installed!${NOCOLOR}")"
        if [ -d "$PROTON_DIRECTORY" ]; then
            echo -e "$(gettext "${GREEN}$PROTON_VERSION is installed!${NOCOLOR}")"
        else
            echo -e "$(gettext "${RED}$PROTON_VERSION is not installed!${NOCOLOR}")"
            exit 1
        fi
    else
        echo -e "$(gettext "${RED}Steam is not installed in $STEAM_DIRECTORY${NOCOLOR}")"
        exit 1
    fi
}

##############################################################################################################################################################################
# CHECK FIREFOX VERSION FOR THE INSTALLER:                                                                                                                                   #
##############################################################################################################################################################################

get_firefox_version() {
    if command -v firefox &>/dev/null; then
        firefox --version | grep -oP '\d+\.\d+(\.\d+)?'
    else
        echo "Firefox is not installed."
    fi
}

    is_snap_firefox_installed() {
    if ! command -v snap &>/dev/null; then
        return 1
    fi
    if snap list 2>/dev/null | grep -q firefox; then
        return 0
    else
        return 1
    fi
    }

check_install_firefox_deb() {
    # Check if Firefox is installed via Snap
    if is_snap_firefox_installed; then
        echo "The installed version of Firefox is from Snap."
        echo "It is recommended to install the DEB version for better performance and compatibility."

        # Prompt user for action
        read -p "Do you want to uninstall the Snap version of Firefox and install the DEB version? (y/n): " choice

        if [[ "$choice" =~ ^[Yy]$ ]]; then
            echo "Proceeding with the uninstallation of the Snap version and installation of the DEB version..."

            # Uninstall Firefox Snap
            sudo snap remove firefox

            # Create an APT keyring directory if it doesn't exist
            sudo install -d -m 0755 /etc/apt/keyrings

            # Import the Mozilla APT repo signing key
            wget -q https://packages.mozilla.org/apt/repo-signing-key.gpg -O- | sudo tee /etc/apt/keyrings/packages.mozilla.org.asc > /dev/null

            # Add Mozilla APT repo to sources.list
            echo "deb [signed-by=/etc/apt/keyrings/packages.mozilla.org.asc] https://packages.mozilla.org/apt mozilla main" | sudo tee /etc/apt/sources.list.d/mozilla.list > /dev/null

            # Set package priority to ensure DEB version is default
            echo '
Package: *
Pin: origin packages.mozilla.org
Pin-Priority: 1000
' | sudo tee /etc/apt/preferences.d/mozilla

            # Update and install Firefox DEB version
            sudo apt update && sudo apt install -y firefox

            echo "Firefox DEB version installed successfully."
        else
            echo "No changes made. Firefox Snap version remains installed."
        fi
    else
        echo "The installed version of Firefox is not from Snap."
    fi
}

PATCH_POPUPS_FILE="/tmp/wine-captionless-popups.patch"
PATCH_POPUPS_URL="$REPO_URL/files/setup/data/wine-captionless-popups.patch"
build_patched_wine() {
    WINE_SOURCE_DIR="$HOME/fusion-wine-source"

    rm -rf "$WINE_BUILD_DIR"
    rm -f "$PATCH_POPUPS_FILE" "$PATCH_PIPE_FILE"
    echo -e "${YELLOW}Building patched Wine for Fusion 360 window fix (this will take 15-30 minutes)...${NOCOLOR}"

    # Download patches
    echo -e "${YELLOW}Downloading Wine patches...${NOCOLOR}"
    curl -L "$PATCH_POPUPS_URL" -o "$PATCH_POPUPS_FILE" || {
        echo -e "${RED}Failed to download Wine patch. Skipping patched build.${NOCOLOR}"
        exit 1
    }

    # Clone Wine source at current installed version
    if [ -d "$WINE_SOURCE_DIR" ]; then
        echo -e "${YELLOW}Existing Wine source directory found, skipping clone.${NOCOLOR}"
    else
        echo -e "${YELLOW}Cloning Wine $WINE_VERSION source...${NOCOLOR}"
        git clone --depth=1 --branch "wine-$WINE_VERSION" \
            https://gitlab.winehq.org/wine/wine.git "$WINE_SOURCE_DIR" || {
            echo -e "${RED}Failed to clone Wine source. Skipping patched build.${NOCOLOR}"
            exit 1
        }
    fi

    # Apply patches
    echo -e "${YELLOW}Applying captionless popup patch...${NOCOLOR}"
    cd "$WINE_SOURCE_DIR"
    if patch -p1 --dry-run < "$PATCH_POPUPS_FILE" >/dev/null 2>&1; then
        patch -p1 < "$PATCH_POPUPS_FILE" || {
            echo -e "${RED}Patch failed to apply. Skipping patched build.${NOCOLOR}"
            exit 1
        }
    elif patch -R -p1 --dry-run < "$PATCH_POPUPS_FILE" >/dev/null 2>&1; then
        echo -e "${YELLOW}Patch already applied. Skipping patch step.${NOCOLOR}"
    else
        echo -e "${RED}Patch does not match this source tree!${NOCOLOR}"
        exit 1
    fi

    # Build and install
    echo -e "${YELLOW}Configuring Wine...${NOCOLOR}"
    rm -rf "$WINE_BUILD_DIR"
    ./configure --prefix="$WINE_BUILD_DIR" CC="ccache gcc" i386_CC="ccache i686-w64-mingw32-gcc" x86_64_CC="ccache x86_64-w64-mingw32-gcc" --enable-archs=i386,x86_64 --disable-tests || exit 1
    echo -e "${YELLOW}Compiling Wine using $(nproc) cores - this will take a while...${NOCOLOR}"
    make -j$(nproc) || exit 1
    make install || exit 1

    # Cleanup
    cd "$HOME"
    read -r -p "Delete source directory $WINE_SOURCE_DIR? [y/N]: " CLEANUP_SOURCE_CHOICE

    case "$CLEANUP_SOURCE_CHOICE" in
        [Yy])
            rm -rf "$WINE_SOURCE_DIR"
            echo -e "${GREEN}Deleted: $WINE_SOURCE_DIR${NOCOLOR}"
            ;;
        *)
            echo -e "${YELLOW}Keeping: $WINE_SOURCE_DIR${NOCOLOR}"
            ;;
    esac

    echo -e "${GREEN}Patched Wine build complete!${NOCOLOR}"
}

PROTON_BUILD_NAME="GE-Proton11-Fusion"
build_patched_proton() {
    PROTON_BUILD_DIR="$HOME/fusion-proton-build"
    PROTON_SOURCE_DIR="$HOME/fusion-proton-source"
    echo -e "${YELLOW}Building patched Proton for Fusion 360 window fix (this will take 15-120 minutes)...${NOCOLOR}"

    # Download patches
    echo -e "${YELLOW}Downloading Proton patches...${NOCOLOR}"
    curl -L "$PATCH_POPUPS_URL" -o "$PATCH_POPUPS_FILE" || {
        echo -e "${RED}Failed to download Proton patch. Skipping patched build.${NOCOLOR}"
        exit 1
    }

    # Clone Proton source
    if [ -d "$PROTON_SOURCE_DIR" ]; then
        echo -e "${YELLOW}Existing Proton source directory found, skipping clone.${NOCOLOR}"
    else
        echo -e "${YELLOW}Cloning Proton source...${NOCOLOR}"
        git clone --recurse-submodules \
            https://github.com/GloriousEggroll/proton-ge-custom.git "$PROTON_SOURCE_DIR" || {
            echo -e "${RED}Failed to clone Proton source. Skipping patched build.${NOCOLOR}"
            exit 1
        }
    fi

    # Apply patches
    echo -e "${YELLOW}Applying captionless popup patch...${NOCOLOR}"
    cd "$PROTON_SOURCE_DIR"
    ./patches/protonprep-valve-staging.sh
    cd "$PROTON_SOURCE_DIR/wine"
    if patch -p1 --dry-run < "$PATCH_POPUPS_FILE" >/dev/null 2>&1; then
        patch -p1 < "$PATCH_POPUPS_FILE" || {
            echo -e "${RED}Patch failed to apply. Skipping patched build.${NOCOLOR}"
            exit 1
        }
    elif patch -R -p1 --dry-run < "$PATCH_POPUPS_FILE" >/dev/null 2>&1; then
        echo -e "${YELLOW}Patch already applied. Skipping patch step.${NOCOLOR}"
    else
        echo -e "${RED}Patch does not match this source tree!${NOCOLOR}"
        exit 1
    fi

    # Build and install
    echo -e "${YELLOW}Configuring Proton...${NOCOLOR}"
    mkdir -p "$PROTON_BUILD_DIR"
    cd "$PROTON_BUILD_DIR"
    "$PROTON_SOURCE_DIR/configure.sh" --build-name="$PROTON_BUILD_NAME" || exit 1

    echo -e "${YELLOW}Compiling Proton using $(nproc) cores - this will take a while...${NOCOLOR}"
    make -j$(nproc) redist || exit 1

    echo -e "${YELLOW}Installing Proton into Steam compatibility tools...${NOCOLOR}"
    if [ -d "$STEAM_COMPAT_DIR" ]; then
        rm -rf "$STEAM_COMPAT_DIR/$PROTON_BUILD_NAME"
        tar -xf "$PROTON_BUILD_DIR/$PROTON_BUILD_NAME.tar.gz" -C "$STEAM_COMPAT_DIR"
        echo -e "${GREEN}Installed $PROTON_BUILD_NAME to Steam compatibility tools!${NOCOLOR}"
    else
        echo -e "${RED}Steam compatibility dir not found: $STEAM_COMPAT_DIR${NOCOLOR}"
        echo -e "${YELLOW}Skipping install and please keep build dir: $PROTON_BUILD_DIR${NOCOLOR}"
    fi

    # Cleanup
    cd "$HOME"
    read -r -p "Delete build directory $PROTON_BUILD_DIR? [y/N]: " CLEANUP_BUILD_CHOICE

    case "$CLEANUP_BUILD_CHOICE" in
        [Yy])
            rm -rf "$PROTON_BUILD_DIR"
            echo -e "${GREEN}Deleted: $PROTON_BUILD_DIR${NOCOLOR}"
            ;;
        *)
            echo -e "${YELLOW}Keeping: $PROTON_BUILD_DIR${NOCOLOR}"
            ;;
    esac

    read -r -p "Delete source directory $PROTON_SOURCE_DIR? [y/N]: " CLEANUP_SOURCE_CHOICE

    case "$CLEANUP_SOURCE_CHOICE" in
        [Yy])
            rm -rf "$PROTON_SOURCE_DIR"
            echo -e "${GREEN}Deleted: $PROTON_SOURCE_DIR${NOCOLOR}"
            ;;
        *)
            echo -e "${YELLOW}Keeping: $PROTON_SOURCE_DIR${NOCOLOR}"
            ;;
    esac

    echo -e "${GREEN}Patched Proton build complete!${NOCOLOR}"
}

##############################################################################################################################################################################
# DOWNLOAD THE REQUIRED FILES FOR THE INSTALLER:                                                                                                                             #
##############################################################################################################################################################################

download_files() {
    echo -e "$(gettext "${GREEN}Downloading the required files for the installation ...${NOCOLOR}")"
    sleep 2

    if [[ ! -x "$WINE_BUILD_DIR/bin/wine" && "$SELECTED_OPTION" == "--install-fix" ]]; then
        download_file "fusion-wine-build.tar.gz" "https://github.com/Lolig4/Autodesk-Fusion-360-for-Linux/releases/download/Pre_Build_Wine%2FProton_01.06.26/fusion-wine-build.tar.gz"
        rm -rf "$WINE_BUILD_DIR"
        echo -e "$(gettext "${YELLOW}Extracting Custom Fusion Wine Build...${NOCOLOR}")"
        tar -xf "$SELECTED_DIRECTORY/downloads/fusion-wine-build.tar.gz" -C "$HOME"
    fi
    if [[ ! -x "$PROTON_DIRECTORY/proton" &&"$SELECTED_OPTION" == "--proton" && "$PROTON_VERSION" == "$PROTON_BUILD_NAME" ]]; then
        download_file "$PROTON_BUILD_NAME.tar.gz" "https://github.com/Lolig4/Autodesk-Fusion-360-for-Linux/releases/download/Pre_Build_Wine%2FProton_01.06.26/$PROTON_BUILD_NAME.tar.gz"
        rm -rf "$PROTON_DIRECTORY"
        echo -e "$(gettext "${YELLOW}Extracting Custom Proton Build...${NOCOLOR}")"
        tar -xf "$SELECTED_DIRECTORY/downloads/$PROTON_BUILD_NAME.tar.gz" -C "$STEAM_COMPAT_DIR"
    fi

    download_file "winetricks" "$WINETRICKS_URL" "$SELECTED_DIRECTORY/bin"
    chmod +x "$SELECTED_DIRECTORY/bin/winetricks"

    #download_file "Fusion_24.03.2026.tar.gz" "$AUTODESK_FUSION_INSTALLER_URL"
    #tar -xf "$SELECTED_DIRECTORY/downloads/Fusion_24.03.2026.tar.gz" -C "$SELECTED_DIRECTORY/downloads"
    download_file "FusionClientInstaller.exe" "$AUTODESK_FUSION_INSTALLER_URL"

    download_file "MicrosoftEdgeWebView2RuntimeInstallerX64.exe" "$WEBVIEW2_INSTALLER_URL"
 
    if (( DOWNLOAD_EXTENSIONS )); then
        download_extensions_files
    fi

    download_file "siappdll.dll" "$SIAPPDLL_URL"

    mkdir -p "$SELECTED_DIRECTORY/downloads/$GPU_DRIVER"

    if [[ $GPU_DRIVER == "DXVK" ]]; then
        download_file "DXVK.reg" "$REPO_URL/files/setup/resource/video_driver/DXVK/DXVK.reg" "$SELECTED_DIRECTORY/downloads/DXVK"
    fi
    download_file "NMachineSpecificOptions.xml" "$REPO_URL/files/setup/resource/video_driver/$GPU_DRIVER/NMachineSpecificOptions.xml" "$SELECTED_DIRECTORY/downloads/$GPU_DRIVER"

    download_file "autodesk_fusion.svg" "$REPO_URL/files/setup/resource/graphics/autodesk_fusion.svg" "$SELECTED_DIRECTORY/resources/graphics"
    download_file "Autodesk Fusion.desktop" "$REPO_URL/files/setup/resource/.desktop/Autodesk%20Fusion.desktop" "$SELECTED_DIRECTORY/.desktop"
    download_file "adskidmgr-opener.desktop" "$REPO_URL/files/setup/resource/.desktop/adskidmgr-opener.desktop" "$SELECTED_DIRECTORY/.desktop"
    download_file "swap_desktop_files.sh" "$REPO_URL/files/setup/data/swap_desktop_files.sh" "$SELECTED_DIRECTORY/bin"
    chmod +x "$SELECTED_DIRECTORY/bin/swap_desktop_files.sh"

    download_file "autodesk_fusion_launcher.sh" "$REPO_URL/files/setup/data/autodesk_fusion_launcher.sh" "$SELECTED_DIRECTORY/bin"
    chmod +x "$SELECTED_DIRECTORY/bin/autodesk_fusion_launcher.sh"
    download_file "fix-navbar-flicker.sh" "$REPO_URL/files/setup/data/fix-navbar-flicker.sh" "$SELECTED_DIRECTORY/bin"
    chmod +x "$SELECTED_DIRECTORY/bin/fix-navbar-flicker.sh"
}

download_extensions_files() {
    echo -e "$(gettext "${YELLOW}Downloading the tested extensions for Autodesk Fusion on Linux ...${NOCOLOR}")"
    EXTENSION_FILE_DIRECTORY="$SELECTED_DIRECTORY/downloads/extensions"
    download_file "Ceska_lokalizace_pro_Autodesk_Fusion.exe" \
        "https://www.cadstudio.cz/dl/Ceska_lokalizace_pro_Autodesk_Fusion_360.exe" \
        "$EXTENSION_FILE_DIRECTORY"
    download_file "HP_3DPrinters_for_Fusion360-win64.msi" \
        "$REPO_URL/files/extensions/HP_3DPrinters_for_Fusion360-win64.msi" \
        "$EXTENSION_FILE_DIRECTORY"
    download_file "Markforged_for_Fusion360-win64.msi" \
        "$REPO_URL/files/extensions/Markforged_for_Fusion360-win64.msi" \
        "$EXTENSION_FILE_DIRECTORY"
    download_file "OctoPrint_for_Fusion360-win64.msi" \
        "$REPO_URL/files/extensions/OctoPrint_for_Fusion360-win64.msi" \
        "$EXTENSION_FILE_DIRECTORY"
    download_file "Ultimaker_Digital_Factory-win64.msi" \
        "$REPO_URL/files/extensions/Ultimaker_Digital_Factory-win64.msi" \
        "$EXTENSION_FILE_DIRECTORY"
    echo -e "$(gettext "${GREEN}All tested extensions for Autodesk Fusion on Linux are downloaded!${NOCOLOR}")"
}

download_file() {
    local FILE_NAME="$1"
    local FILE_URL="$2"
    local DESTINATION_DIRECTORY="${3:-$SELECTED_DIRECTORY/downloads/}"
    local FILE="$DESTINATION_DIRECTORY/$FILE_NAME"

    if [ -f "$FILE" ]; then
        echo -e "$(gettext "${GREEN}$FILE_NAME exists!${NOCOLOR}")"
        if find "$FILE" -mtime +7 | grep -q .; then
            echo -e "$(gettext "${YELLOW}$FILE_NAME exists but is older than 7 days and will be updated!")${NOCOLOR}"
            rm -rf "$FILE"
            curl -L "$FILE_URL" -o "$FILE"
        fi
    else
        echo -e "$(gettext "${YELLOW}$FILE_NAME doesn't exist and will be downloaded for you!${NOCOLOR}")"
        curl -L "$FILE_URL" -o "$FILE"
    fi
    
}

##############################################################################################################################################################################
# CHECK AND INSTALL WINE FOR THE INSTALLER:                                                                                                                                  #
##############################################################################################################################################################################

check_and_install_wine() {
    # Check if wine is installed
    if [ -x "$(command -v wine)" ]; then
        echo "Wine is installed!"
        WINE_VERSION="$(wine --version  | cut -d ' ' -f1 | sed -e 's/wine-//' -e 's/-rc.*//')"
        WINE_VERSION_MAJOR_RELEASE="$(echo $WINE_VERSION | cut -d '.' -f1)"
        WINE_VERSION_MINOR_RELEASE="$(echo $WINE_VERSION | cut -d '.' -f2)"
        
        # Check if the installed wine version is at least 11.1 or higher (wine_version_series and wine_version_series_release)
        if [ "$WINE_VERSION_MAJOR_RELEASE" -gt 11 ] || ([ "$WINE_VERSION_MAJOR_RELEASE" -eq 11 ] && [ "$WINE_VERSION_MINOR_RELEASE" -ge 1 ]); then
            echo "Wine version $WINE_VERSION is installed!"
            WINE_STATUS=1
        else
            echo "Wine version $WINE_VERSION is installed, but this version is too old and will be updated for you!"
            WINE_STATUS=0
        fi

    else
        echo ${YELLOW}"Wine is not installed on your system and will be installed for you!"
        WINE_STATUS=0
    fi

    # Check wine status 0 and install Wine version 
    if (( !WINE_STATUS )); then
        # Check which Linux Distro is used; fall back to /etc/os-release on Arch and others without lsb_release
        if command -v lsb_release &>/dev/null; then
            DISTRO_VERSION=$(lsb_release -ds)
        else
            # shellcheck source=/dev/null
            . /etc/os-release
            DISTRO_VERSION="${NAME} ${VERSION:-}"
        fi
        if [[ $DISTRO_VERSION == *"Arch"*"Linux"* ]] || [[ $DISTRO_VERSION == *"Manjaro"*"Linux"* ]] || [[ $DISTRO_VERSION == *"EndeavourOS"* ]] || [[ $DISTRO_VERSION == *"CachyOS"* ]]; then
            echo "${GREEN}Installing Wine for Arch Linux ...${NOCOLOR}"
            if grep -q '^\[multilib\]$' /etc/pacman.conf; then
                echo "Multilib is already enabled!"
                    sudo pacman -R wine wine-mono wine_gecko winetricks --noconfirm
                    sudo pacman -Syu --needed wine wine-mono wine_gecko winetricks
            else
                echo "Enabling Multilib ..."
                    echo -e "[multilib]\nInclude = /etc/pacman.d/mirrorlist" >> /etc/pacman.conf
                    sudo pacman -R wine wine-mono wine_gecko winetricks --noconfirm
                    sudo pacman -Syu --needed wine wine-mono wine_gecko winetricks
            fi
        elif [[ $DISTRO_VERSION == *"Debian"*"12"* ]]; then
            echo "${GREEN}Installing Wine for Debian 12 ...${NOCOLOR}"
                sudo apt --allow-releaseinfo-change update
                sudo dpkg --add-architecture i386
                sudo rm /etc/apt/sources.list.d/wine* /etc/apt/sources.list.d/*wine* 2>/dev/null
                sudo mkdir -pm755 /etc/apt/keyrings
                wget -O - https://dl.winehq.org/wine-builds/winehq.key | sudo gpg --dearmor -o /etc/apt/keyrings/winehq-archive.key -
                sudo wget -NP /etc/apt/sources.list.d/ https://dl.winehq.org/wine-builds/debian/dists/bookworm/winehq-bookworm.sources
                sudo apt update
                sudo apt remove wine* --purge
                sudo apt autoremove -y
                sudo apt install -y --install-recommends winehq-staging
        elif [[ $DISTRO_VERSION == *"Debian"*"13"* ]]; then
            echo "${GREEN}Installing Wine for Debian 13 ...${NOCOLOR}"
                sudo apt --allow-releaseinfo-change update
                sudo dpkg --add-architecture i386
                sudo rm /etc/apt/sources.list.d/wine* /etc/apt/sources.list.d/*wine* 2>/dev/null
                sudo mkdir -pm755 /etc/apt/keyrings
                wget -O - https://dl.winehq.org/wine-builds/winehq.key | sudo gpg --dearmor -o /etc/apt/keyrings/winehq-archive.key -
                sudo wget -NP /etc/apt/sources.list.d/ https://dl.winehq.org/wine-builds/debian/dists/trixie/winehq-trixie.sources
                sudo apt update
                sudo apt remove wine* --purge
                sudo apt autoremove -y
                sudo apt install -y --install-recommends winehq-staging
        elif [[ $DISTRO_VERSION == *"Debian"*"Testing"* ]] || [[ $DISTRO_VERSION == *"Debian"*"testing"* ]]; then
            echo "${GREEN}Installing Wine for Debian testing ...${NOCOLOR}"
                sudo apt --allow-releaseinfo-change update
                sudo dpkg --add-architecture i386
                sudo rm /etc/apt/sources.list.d/wine* /etc/apt/sources.list.d/*wine* 2>/dev/null
                sudo mkdir -pm755 /etc/apt/keyrings
                wget -O - https://dl.winehq.org/wine-builds/winehq.key | sudo gpg --dearmor -o /etc/apt/keyrings/winehq-archive.key -
                sudo wget -NP /etc/apt/sources.list.d/ https://dl.winehq.org/wine-builds/debian/dists/trixie/winehq-trixie.sources
                sudo apt update
                sudo apt remove wine* --purge
                sudo apt autoremove -y
                sudo apt install -y --install-recommends winehq-staging
        elif [[ $DISTRO_VERSION == *"Ubuntu"*"20.04"* ]] || [[ $DISTRO_VERSION == *"Linux"*"Mint"*"20"* ]] || [[ $DISTRO_VERSION == *"Pop"*"OS"*"20.04"* ]] || [[ $DISTRO_VERSION == *"pop"*"20.04"* ]]; then
            echo "${GREEN}Installing Wine for Ubuntu 20.04 ...${NOCOLOR}"
                sudo dpkg --add-architecture i386
                sudo rm /etc/apt/sources.list.d/wine* /etc/apt/sources.list.d/*wine* 2>/dev/null
                sudo apt-key list | grep -A 2 "wine" | grep "pub" | awk "{print \$2}" | cut -d"/" -f2 | xargs -r apt-key del
                sudo mkdir -pm755 /etc/apt/keyrings
                wget -O /etc/apt/keyrings/winehq-archive.key https://dl.winehq.org/wine-builds/winehq.key
                sudo wget -NP /etc/apt/sources.list.d/ https://dl.winehq.org/wine-builds/ubuntu/dists/focal/winehq-focal.sources
                sudo apt update
                sudo apt remove wine* --purge
                sudo apt autoremove -y
                sudo apt install -y --install-recommends winehq-staging
        elif [[ $DISTRO_VERSION == *"Ubuntu"*"22.04"* ]] || [[ $DISTRO_VERSION == *"Linux"*"Mint"*"21"* ]] || [[ $DISTRO_VERSION == *"Pop"*"22.04"* ]] || [[ $DISTRO_VERSION == *"Zorin"*"17"* ]]; then
            echo "${GREEN}Installing Wine for Ubuntu 22.04 ...${NOCOLOR}"
                sudo dpkg --add-architecture i386
                sudo rm /etc/apt/sources.list.d/wine* /etc/apt/sources.list.d/*wine* 2>/dev/null
                sudo mkdir -pm755 /etc/apt/keyrings
                wget -O - https://dl.winehq.org/wine-builds/winehq.key | sudo gpg --dearmor -o /etc/apt/keyrings/winehq-archive.key -
                sudo wget -NP /etc/apt/sources.list.d/ https://dl.winehq.org/wine-builds/ubuntu/dists/jammy/winehq-jammy.sources
                sudo apt update
                sudo apt remove wine* --purge
                sudo apt autoremove -y
                sudo apt install -y --install-recommends winehq-staging
        elif [[ $DISTRO_VERSION == *"Ubuntu"*"24.04"* ]] || [[ $DISTRO_VERSION == *"Linux"*"Mint"*"22"* ]] || [[ $DISTRO_VERSION == *"Pop"*"24.04"* ]] || [[ $DISTRO_VERSION == *"Zorin"*"18"* ]]; then
            echo "${GREEN}Installing Wine for Ubuntu 24.04 ...${NOCOLOR}"
                sudo dpkg --add-architecture i386
                sudo rm /etc/apt/sources.list.d/wine* /etc/apt/sources.list.d/*wine* 2>/dev/null
                sudo mkdir -pm755 /etc/apt/keyrings
                wget -O - https://dl.winehq.org/wine-builds/winehq.key | sudo gpg --dearmor -o /etc/apt/keyrings/winehq-archive.key -
                sudo wget -NP /etc/apt/sources.list.d/ https://dl.winehq.org/wine-builds/ubuntu/dists/noble/winehq-noble.sources
                sudo apt update
                sudo apt remove wine* --purge
                sudo apt autoremove -y
                sudo apt install -y --install-recommends winehq-staging
        elif [[ $DISTRO_VERSION == *"Ubuntu"*"25.04"* ]]; then
            echo "${GREEN}Installing Wine for Ubuntu 25.04 ...${NOCOLOR}"
                sudo dpkg --add-architecture i386
                sudo rm /etc/apt/sources.list.d/wine* /etc/apt/sources.list.d/*wine* /etc/apt/keyrings/wine*.key 2>/dev/null
                sudo mkdir -pm755 /etc/apt/keyrings
                wget -O - https://dl.winehq.org/wine-builds/winehq.key | sudo gpg --dearmor -o /etc/apt/keyrings/winehq-archive.key -
                sudo wget -NP /etc/apt/sources.list.d/ https://dl.winehq.org/wine-builds/ubuntu/dists/plucky/winehq-plucky.sources
                sudo apt update
                sudo apt remove wine* --purge
                sudo apt autoremove -y
                sudo apt install -y --install-recommends winehq-staging
            echo "${GREEN}Installation complete.${NOCOLOR}"
            echo "${RED}WARNING! 25.04 deprecated, WINEHQ=11.1. Problems might arise downstream, it is recommended to upgrade to 25.10...${NOCOLOR}"
            sleep 5
        elif [[ $DISTRO_VERSION == *"Ubuntu"*"25.10"* ]]; then
            echo "${GREEN}Installing Wine for Ubuntu 25.10 ...${NOCOLOR}"
                sudo rm /etc/apt/sources.list.d/wine* /etc/apt/sources.list.d/*wine* /etc/apt/keyrings/wine*.key 2>/dev/null
                sudo mkdir -pm755 /etc/apt/keyrings
                wget -O - https://dl.winehq.org/wine-builds/winehq.key | sudo gpg --dearmor -o /etc/apt/keyrings/winehq-archive.key -
                sudo wget -NP /etc/apt/sources.list.d/ https://dl.winehq.org/wine-builds/ubuntu/dists/questing/winehq-questing.sources
                sudo apt update
                sudo apt remove wine* --purge
                sudo apt autoremove -y
                sudo apt install --install-recommends winehq-staging -y
        elif [[ $DISTRO_VERSION == *"Fedora"* && $DISTRO_VERSION == *"43"* ]] || [[ $DISTRO_VERSION == *"Nobara"* ]]; then
            echo "Installing Wine for Fedora 43 ..."
            echo -e "$(gettext "${YELLOW}Multiple Wine repos detected. Please choose which to use:${NOCOLOR}")"
            echo "1) WineHQ Repository"
            echo "2) openSUSE-Wine-OBS Repository"
            read -p "Enter your choice (1 or 2): " wine_repo_choice

            case $wine_repo_choice in
                1)
                    echo -e "$(gettext "${GREEN}WineHQ Repository selected. The WineHQ Repository will be used for the installation.${NOCOLOR}")"
                        sudo dnf config-manager addrepo --from-repofile=https://dl.winehq.org/wine-builds/fedora/43/winehq.repo
                        sudo dnf remove -y wine wine-*
                        sudo dnf install -y winehq-staging
                    ;;
                2)
                    echo -e "$(gettext "${GREEN}openSUSE-Wine-OBS Repository selected. The openSUSE-Wine-OBS Repository will be used for the installation.${NOCOLOR}")"
                        sudo rpm --import https://download.opensuse.org/repositories/Emulators:/Wine:/Fedora/Fedora_43/repodata/repomd.xml.key
                        sudo dnf config-manager addrepo --from-repofile=https://download.opensuse.org/repositories/Emulators:/Wine:/Fedora/Fedora_43/Emulators:Wine:Fedora.repo
                        sudo dnf remove -y wine wine-*
                        sudo dnf install -y winehq-staging
                    ;;
                *)
                    echo -e "$(gettext "${RED}Invalid choice. The WineHQ Repository will be used for the installation.${NOCOLOR}")"
                        sudo dnf config-manager addrepo --from-repofile=https://dl.winehq.org/wine-builds/fedora/43/winehq.repo
                        sudo dnf remove -y wine wine-*
                        sudo dnf install -y winehq-staging
                    ;;
            esac
        elif [[ $DISTRO_VERSION == *"Fedora"*"Rawhide"* ]]; then
            echo "Installing Wine for Fedora rawhide ..."
                sudo dnf config-manager addrepo --from-repofile=https://download.opensuse.org/repositories/Emulators:/Wine:/Fedora/Fedora_Rawhide/Emulators:Wine:Fedora.repo
                sudo dnf remove wine wine-*
                sudo dnf install -y winehq-staging
        elif [[ $DISTRO_VERSION == *"Gentoo"* ]]; then
            #change to sudo
            echo "Installing Wine for Gentoo ..."
            pkexec emerge -av app-emulation/wine
        elif [[ $DISTRO_VERSION == *"openSUSE"*"15.6"* ]]; then
            #change to sudo
            echo "Installing Wine for openSUSE 15.6 ..."
            pkexec bash -c '
                repos=$(zypper repos --uri | grep wine | awk '{print $1}')
                # Remove each identified repository
                for repo in $repos; do
                    echo "Removing repository: $repo"
                    zypper removerepo "$repo"
                done
                zypper addrepo -cfp 90 "https://download.opensuse.org/repositories/Emulators:/Wine/15.6/" wine
                zypper refresh
                zypper remove wine wine-* winetricks --no-confirm
                zypper install -y wine'
        elif [[ $DISTRO_VERSION == *"openSUSE"*"16.0"* ]]; then
            #change to sudo
            echo "Installing Wine for openSUSE 16.0 ..."
            pkexec bash -c '
                repos=$(zypper repos --uri | grep wine | awk '{print $1}')
                # Remove each identified repository
                for repo in $repos; do
                    echo "Removing repository: $repo"
                    zypper removerepo "$repo"
                done
                zypper addrepo -cfp 90 "https://download.opensuse.org/repositories/Emulators:/Wine/16.0/" wine
                zypper refresh
                zypper remove wine wine-* winetricks --no-confirm
                zypper install -y wine'
        elif [[ $DISTRO_VERSION == *"openSUSE"*"Tumbleweed"* ]]; then
            #change to sudo
            echo "Installing Wine for openSUSE tumbleweed ..."
            pkexec bash -c '
                repos=$(zypper repos --uri | grep wine | awk '{print $1}')
                # Remove each identified repository
                for repo in $repos; do
                    echo "Removing repository: $repo"
                    zypper removerepo "$repo"
                done
                zypper addrepo -cfp 90 "https://download.opensuse.org/repositories/Emulators:/Wine/openSUSE_Tumbleweed/" wine
                zypper refresh
                zypper remove wine wine-* winetricks --no-confirm
                zypper install -y wine'
        elif [[ $DISTRO_VERSION == *"Red"*"Hat"*"Enterprise"*"Linux"* ]] || [[ $DISTRO_VERSION == *"Alma"*"Linux"* ]] || [[ $DISTRO_VERSION == *"Rocky"*"Linux"* ]]; then
            #change to sudo
            echo "Installing Wine for RHEL 9, 10, ..."
            if command -v dnf &> /dev/null; then # Use dnf for newer distributions
                pkexec bash -c '
                    dnf -y groupinstall 'Development Tools'
                    dnf -y install gcc libX11-devel freetype-devel zlib-devel libxcb-devel libxslt-devel
                    curl -L https://dl.winehq.org/wine/source/11.x/wine-11.1.tar.xz -o /tmp/wine-11.1.tar.xz
                    tar -xvf /tmp/wine-11.1.tar.xz -C /tmp/
                    ./tmp/wine-11.1/configure --enable-win64
                    make -C /tmp/wine-11.1
                    make -C /tmp/wine-11.1 install'
            else  # Use yum for older distributions
                pkexec bash -c '
                    yum -y groupinstall 'Development Tools'
                    yum install gcc libX11-devel freetype-devel zlib-devel libxcb-devel libxslt-devel
                    curl -L https://dl.winehq.org/wine/source/11.x/wine-11.1.tar.xz -o /tmp/wine-11.1.tar.xz
                    tar -xvf /tmp/wine-11.1.tar.xz -C /tmp/
                    ./tmp/wine-11.1/configure --enable-win64
                    make -C /tmp/wine-11.1
                    make -C /tmp/wine-11.1 install'
            fi
        elif [[ $DISTRO_VERSION == *"Solus"* ]]; then
            #change to sudo
            echo "Installing Wine for Solus ..."
            pkexec eopkg install -y winehq-staging
        elif [[ $DISTRO_VERSION == *"Void"* ]]; then
            #change to sudo
            echo "Installing Wine for Void Linux ..."
            pkexec xbps-install -Syu --yes wine
        elif [[ $DISTRO_VERSION == *"NixOS"* ]] || [[ $DISTRO_VERSION == *"nixos"* ]]; then
            #change to sudo
            echo "Installing Wine for NixOS ..."
            pkexec nix-env -iA nixos.wine nixos.winetricks --yes
        # Add more distributions and versions here ...
        # elif ...
        else
            echo "Error: Your Linux distribution and version are not supported."
        fi
    fi
}

# Load the icons and .desktop-files:
autodesk_fusion_shortcuts_load() {
    if [ -d "$FUSION_DESKTOP_DIRECTORY" ]; then
        local -A EXISTING_IDS=()
        for DIR in "$FUSION_DESKTOP_DIRECTORY/"*; do
            if [ ! -d "$DIR" ]; then
                continue
            fi
            local NAME
            NAME="$(basename "$DIR")"
            if [[ "$NAME" =~ ^[0-9]+$ ]]; then
                EXISTING_IDS["$NAME"]=1
                    # Deactivate .desktop files in this directory
                    if [ -f "$DIR/Autodesk Fusion.desktop" ]; then
                        mv -f "$DIR/Autodesk Fusion.desktop" "$DIR/Autodesk Fusion.desktop.bak"
                    fi
                    if [ -f "$DIR/adskidmgr-opener.desktop" ]; then
                        mv -f "$DIR/adskidmgr-opener.desktop" "$DIR/adskidmgr-opener.desktop.bak"
                    fi
            fi
        done
        local NEW_ID=1
        while [[ -n "${EXISTING_IDS[$NEW_ID]+x}" ]]; do
            (( NEW_ID++ ))
        done
    else
        local NEW_ID=1
    fi

    local SCHORTCUT_DIRECTORY="$FUSION_DESKTOP_DIRECTORY/$NEW_ID"
    mkdir -p "$SCHORTCUT_DIRECTORY"
    rm -f "$FUSION_DESKTOP_DIRECTORY/Autodesk Fusion.desktop" # Is Necessary!
    rm -f "$FUSION_DESKTOP_DIRECTORY/adskidmgr-opener.desktop" # Clean up old desktop files from older versions of the installer.
    rm -f "$DESKTOP_DIRECTORY/adskidmgr-opener.desktop" # Clean up old desktop files from older versions of the installer.
    rm -f "$DESKTOP_DIRECTORY/wine-Programs-Autodesk-1-Autodesk Fusion.desktop" # I Dont know if this file is ever created.
    rm -f "$DESKTOP_DIRECTORY/wine-extension-cam360.desktop"
    rm -f "$DESKTOP_DIRECTORY/wine-extension-f2d.desktop"
    rm -f "$DESKTOP_DIRECTORY/wine-extension-f2t.desktop"
    rm -f "$DESKTOP_DIRECTORY/wine-extension-f3d.desktop"
    rm -f "$DESKTOP_DIRECTORY/wine-extension-f3z.desktop"
    rm -f "$DESKTOP_DIRECTORY/wine-extension-fbrd.desktop"
    rm -f "$DESKTOP_DIRECTORY/wine-extension-flbr.desktop"
    rm -f "$DESKTOP_DIRECTORY/wine-extension-fsch.desktop"
    rm -f "$DESKTOP_DIRECTORY/wine-protocol-fusion360.desktop"

    echo "$SELECTED_DIRECTORY" >> "$SCHORTCUT_DIRECTORY/location.log"
    chmod 444 "$SCHORTCUT_DIRECTORY/location.log"

    # Create a .desktop file (launcher.sh) for Autodesk Fusion!
    cp "$SELECTED_DIRECTORY/.desktop/Autodesk Fusion.desktop" "$SCHORTCUT_DIRECTORY/Autodesk Fusion.desktop"
    echo "Exec=$SELECTED_DIRECTORY/bin/autodesk_fusion_launcher.sh" >> "$SCHORTCUT_DIRECTORY/Autodesk Fusion.desktop"
    echo "Path=$SELECTED_DIRECTORY/bin" >> "$SCHORTCUT_DIRECTORY/Autodesk Fusion.desktop"

    # Set the permissions for the .desktop file to read-only
    chmod 444 "$SCHORTCUT_DIRECTORY/Autodesk Fusion.desktop"

    #Create mimetype link to handle web login call backs to the Identity Manager
    cp "$SELECTED_DIRECTORY/.desktop/adskidmgr-opener.desktop" "$SCHORTCUT_DIRECTORY/adskidmgr-opener.desktop"
    if [[ "$SELECTED_OPTION" == "--install-fix" ]]; then
        echo "Exec=sh -c 'env WINEPREFIX=$WINE_PFX WINESERVER="$WINESERVER" "$WINE" \"\$(find $WINE_PFX -name AdskIdentityManager.exe | head -1)\" \"%u\"'" >> "$SCHORTCUT_DIRECTORY/adskidmgr-opener.desktop"
    elif [[ "$SELECTED_OPTION" == "--proton" ]]; then
        echo "Exec=sh -c 'env STEAM_COMPAT_CLIENT_INSTALL_PATH="$STEAM_DIRECTORY" STEAM_COMPAT_DATA_PATH="$PROTONPREFIX_DIRECTORY" "$PROTON_DIRECTORY/proton" run \"\$(find $WINE_PFX -name AdskIdentityManager.exe | head -1)\" \"%u\"'" >> "$SCHORTCUT_DIRECTORY/adskidmgr-opener.desktop"
    else
        echo "Exec=sh -c 'env WINEPREFIX=$WINE_PFX "$WINE" \"\$(find $WINE_PFX -name AdskIdentityManager.exe | head -1)\" \"%u\"'" >> "$SCHORTCUT_DIRECTORY/adskidmgr-opener.desktop"
    fi

    #Set the permissions for the .desktop file to read-only
    chmod 444 "$SCHORTCUT_DIRECTORY/adskidmgr-opener.desktop"
    
    update-desktop-database "$DESKTOP_DIRECTORY" 2>/dev/null || true
    #Set the mimetype handler for the Identity Manager
    xdg-mime default adskidmgr-opener.desktop x-scheme-handler/adskidmgr 2>/dev/null
}

###############################################################################################################################################################
# Execute the installation of Autodesk Fusion                                                                                                                 #
###############################################################################################################################################################
autodesk_fusion_run_install_client() {
    echo -e "$(gettext "${YELLOW}Installing Autodesk Fusion 360 Client ...${NOCOLOR}")"
    sleep 2
    timeout -k 10m 9m "$WINE" "$WIN_DOWNLOADS_DIRECTORY/FusionClientInstaller.exe" --quiet 2>> "$SELECTED_DIRECTORY/logs/FusionClientInstaller_1.log"
    sleep 5
    echo -e "$(gettext "${YELLOW}Finalizing Autodesk Fusion 360 installation...${NOCOLOR}")"
    timeout -k 5m 1m "$WINE" "$WIN_DOWNLOADS_DIRECTORY/FusionClientInstaller.exe" --quiet 2>> "$SELECTED_DIRECTORY/logs/FusionClientInstaller_2.log"
    echo -e "$(gettext "${GREEN}Autodesk Fusion 360 Client installation completed!${NOCOLOR}")"
}

#################################################################################################################################################################
# Add/Patch the siappdll.dll to fix the SpaceMouse issue                                                                                                        #
#################################################################################################################################################################
autodesk_fusion_patch_siappdll() {
    echo -e "${YELLOW}Patching the siappdll.dll file for Autodesk Fusion ...${NOCOLOR}"
    sleep 2

    SIAPPDLL=$(find "$WINE_PFX" -name 'Qt6WebEngineCore.dll' -printf "%T+ %p\n" | sort -r | head -n 1 | sed -r 's/^[^ ]+ //')
    SIAPPDLL_DIR=$(dirname "$SIAPPDLL")
    
    # Check if the siappdll.dll file exists before attempting to backup
    if [ -f "$SIAPPDLL_DIR/siappdll.dll" ]; then
        # Backup the siappdll.dll file
        cp -f "$SIAPPDLL_DIR/siappdll.dll" "$SIAPPDLL_DIR/siappdll.dll.bak"
        echo -e "${GREEN}The siappdll.dll file is backed up as siappdll.dll.bak!${NOCOLOR}"
    else
        echo -e "${RED}The siappdll.dll file does not exist. No backup was made.${NOCOLOR}"
    fi

    # Copy the patched siappdll.dll file to the Autodesk Fusion directory
    cp -f "$SELECTED_DIRECTORY/downloads/siappdll.dll" "$SIAPPDLL_DIR/siappdll.dll"
    echo -e "${GREEN}The siappdll.dll file is patched successfully!${NOCOLOR}"
}

#################################################################################################################################################################
# Wine configuration for Autodesk Fusion                                                                                                                        #
#################################################################################################################################################################
wine_autodesk_fusion_install() {
    # Note that the winetricks sandbox verb merely removes the desktop integration and Z: drive symlinks and is not a "true" sandbox.
    # It protects against errors rather than malice. It's useful for, e.g., keeping games from saving their settings in random subdirectories of your home directory.
    # But it still ensures that wine, for example, no longer has access permissions to Home!
    # For this reason, the EXE files must be located directly in the Wineprefix folder!

    WINE="wine"
    WINESERVER="wineserver"
    WINETRICKS="$SELECTED_DIRECTORY/bin/winetricks"
    export WINEPREFIX="$WINE_PFX"

    if [[ "$SELECTED_OPTION" == "--install-fix" ]]; then
        WINE="$WINE_BUILD_DIR/bin/wine"
        WINESERVER="$WINE_BUILD_DIR/bin/wineserver"
        export WINE WINESERVER

        "$WINE_BUILD_DIR/bin/wineboot" --init
    elif [[ "$SELECTED_OPTION" == "--proton" ]]; then
        echo -e "$(gettext "${YELLOW}Init Proton...${NOCOLOR}")"
        if ! pgrep -x steam >/dev/null 2>&1; then
            echo -e "$(gettext "${YELLOW}Starting Steam (background, no window)...${NOCOLOR}")"
            # Start Steam in a separate user scope to avoid a parent-child link.
            if command -v systemd-run >/dev/null 2>&1; then
                setsid -f systemd-run --user --scope --quiet steam -silent </dev/null >/dev/null 2>&1
            else
                # Fallback if systemd-run is not available; Steam is linked to Fusion, so it can look like Fusion never exited.
                setsid -f steam -silent </dev/null >/dev/null 2>&1
            fi
            sleep 5
        fi
        USER="steamuser"
        WINE="$PROTON_DIRECTORY/files/bin/wine"
        WINESERVER="$PROTON_DIRECTORY/files/bin/wineserver"
        export WINE WINESERVER

        STEAM_COMPAT_CLIENT_INSTALL_PATH="$STEAM_DIRECTORY" \
        STEAM_COMPAT_DATA_PATH="$PROTONPREFIX_DIRECTORY" \
        "$PROTON_DIRECTORY/proton" run wineboot --init
    else
        wineboot --init
    fi

    "$WINESERVER" -w

    echo -e "$(gettext "${YELLOW}Setting up the Wine prefix for Autodesk Fusion 360 in Sandbox... (suppressed)${NOCOLOR}")"
    "$WINETRICKS" -q sandbox >> "$SELECTED_DIRECTORY/logs/winetricks_sandbox.log" 2>&1

    echo -e "$(gettext "${YELLOW}Linking the downloads folder to the Wine prefix...${NOCOLOR}")"
    WIN_DOWNLOADS_DIRECTORY="$WINE_PFX/drive_c/users/$USER/Downloads"
    rm -rf "$WIN_DOWNLOADS_DIRECTORY"
    ln -s "$SELECTED_DIRECTORY/downloads" "$WIN_DOWNLOADS_DIRECTORY"

    echo -e "$(gettext "${YELLOW}Configuring the Wine prefix for Autodesk Fusion 360...${NOCOLOR}")"
    # We must install some packages! (dotnet20 is needed, because of https://bugs.winehq.org/show_bug.cgi?id=41727#c5)
    "$WINETRICKS" -q atmlib gdiplus corefonts cjkfonts dotnet20 dotnet48 msxml4 msxml6 vcrun2022 fontsmooth=rgb winhttp win10 2>> "$SELECTED_DIRECTORY/logs/winetricks_dotnet48.log"
    # We must install cjkfonts again then sometimes it doesn't work in the first time!
    echo -e "$(gettext "${YELLOW}Re-installing cjkfonts... (suppressed)${NOCOLOR}")"
    "$WINETRICKS" -q cjkfonts >> "$SELECTED_DIRECTORY/logs/winetricks_cjkfonts_2.log" 2>&1
    # We must set to Windows 11 again because some other winetricks sometimes set it back to Windows XP!
    echo -e "$(gettext "${YELLOW}Setting Windows 11 as the Windows version... (suppressed)${NOCOLOR}")"
    "$WINETRICKS" -q win11 >> "$SELECTED_DIRECTORY/logs/winetricks_win11.log" 2>&1
    # Remove tracking metrics/calling home
    "$WINE" REG ADD "HKCU\Software\Wine\DllOverrides" /v "adpclientservice.exe" /t REG_SZ /d native /f
    # Navigation bar does not work well with anything other than the wine builtin DX9
    "$WINE" REG ADD "HKCU\Software\Wine\DllOverrides" /v "AdCefWebBrowser.exe" /t REG_SZ /d builtin /f
    # Use Visual Studio Redist that is bundled with the application
    "$WINE" REG ADD "HKCU\Software\Wine\DllOverrides" /v "msvcp140" /t REG_SZ /d native /f
    "$WINE" REG ADD "HKCU\Software\Wine\DllOverrides" /v "mfc140u" /t REG_SZ /d native /f
    # Fixed the problem with the bcp47langs issue and now the login works again!  ## Schould work with wine 11.0 and newer, without this fix, but i leave it for proton.
    "$WINE" REG ADD "HKCU\Software\Wine\DllOverrides" /v "bcp47langs" /t REG_SZ /d "" /f
    "$WINE" REG ADD "HKCU\Software\Wine\X11 Driver" /v "Managed" /t REG_SZ /d "Y" /f
    "$WINE" REG ADD "HKCU\Software\Wine\X11 Driver" /v "Decorated" /t REG_SZ /d "Y" /f
    # For WebView2installer -v 109
    echo -e "$(gettext "${YELLOW}Installing Microsoft Edge WebView2 Runtime for Autodesk Fusion ...${NOCOLOR}")"
    "$WINE" "$WIN_DOWNLOADS_DIRECTORY/MicrosoftEdgeWebView2RuntimeInstallerX64.exe" /silent /install 2>> "$SELECTED_DIRECTORY/logs/WebView2_install.log"
    echo -e "$(gettext "${GREEN}Microsoft Edge WebView2 Runtime installation completed!${NOCOLOR}")"
    APPDATA_DIRECTORY="$WINE_PFX/drive_c/users/$USER/AppData"
    APPLICATION_DATA_DIRECTORY="$WINE_PFX/drive_c/users/$USER/Application Data"
    mkdir -p "$APPDATA_DIRECTORY/Roaming/Microsoft/Internet Explorer/Quick Launch/User Pinned"

    if [[ $GPU_DRIVER = "DXVK" ]]; then
        "$WINETRICKS" -q dxvk
        "$WINE" regedit.exe "C:\\users\\$USER\\Downloads\\DXVK\\DXVK.reg"
    fi
    autodesk_fusion_run_install_client
    mkdir -p "$APPDATA_DIRECTORY/Roaming/Autodesk/Neutron Platform/Options"
    mkdir -p "$APPDATA_DIRECTORY/Local/Autodesk/Neutron Platform/Options"
    mkdir -p "$APPLICATION_DATA_DIRECTORY/Autodesk/Neutron Platform/Options"
    cp "$SELECTED_DIRECTORY/downloads/$GPU_DRIVER/NMachineSpecificOptions.xml" "$APPDATA_DIRECTORY/Roaming/Autodesk/Neutron Platform/Options/NMachineSpecificOptions.xml" || return
    cp "$SELECTED_DIRECTORY/downloads/$GPU_DRIVER/NMachineSpecificOptions.xml" "$APPDATA_DIRECTORY/Local/Autodesk/Neutron Platform/Options/NMachineSpecificOptions.xml" || return
    cp "$SELECTED_DIRECTORY/downloads/$GPU_DRIVER/NMachineSpecificOptions.xml" "$APPLICATION_DATA_DIRECTORY/Autodesk/Neutron Platform/Options/NMachineSpecificOptions.xml" || return
}

###############################################################################################################################################################

# Check and install the selected extensions
wine_autodesk_fusion_install_extensions() {
    if [[ "$SELECTED_EXTENSIONS" == *"CzechlocalizationforF360"* ]]; then
        run_install_extension_client "Ceska_lokalizace_pro_Autodesk_Fusion.exe"
    fi
    if [[ "$SELECTED_EXTENSIONS" == *"HP3DPrintersforAutodesk®Fusion®"* ]]; then
        run_install_extension_client "HP_3DPrinters_for_Fusion360-win64.msi"
    fi
    if [[ "$SELECTED_EXTENSIONS" == *"MarkforgedforAutodesk®Fusion®"* ]]; then
        run_install_extension_client "Markforged_for_Fusion360-win64.msi"
    fi
    if [[ "$SELECTED_EXTENSIONS" == *"OctoPrintforAutodesk®Fusion360™"* ]]; then
        run_install_extension_client "OctoPrint_for_Fusion360-win64.msi"
    fi
    if [[ "$SELECTED_EXTENSIONS" == *"UltimakerDigitalFactoryforAutodeskFusion360™"* ]]; then
        run_install_extension_client "Ultimaker_Digital_Factory-win64.msi"
    fi
}

run_install_extension_client() {
    local EXTENSION_FILE="$1"
    local WIN_EXTENSION_DIRECTORY="C:\\users\\$USER\\Downloads\\extensions"
    if [[ "$EXTENSION_FILE" == *.msi ]]; then
        "$WINE" msiexec /i "$WIN_EXTENSION_DIRECTORY\\$EXTENSION_FILE" /quiet
    else
        "$WINE" "$WIN_DOWNLOADS_DIRECTORY/$EXTENSION_FILE"
    fi
}

###############################################################################################################################################################

autodesk_fusion_safe_logfile() {
    # Log the Wineprefixes
    echo "$GPU_DRIVER" >> "$SELECTED_DIRECTORY/logs/wineprefixes.log"
    echo "$SELECTED_DIRECTORY" >> "$SELECTED_DIRECTORY/logs/wineprefixes.log"
    echo "$WINE_PFX" >> "$SELECTED_DIRECTORY/logs/wineprefixes.log"
    if [[ "$SELECTED_OPTION" == "--install-fix" ]]; then
        echo "Wine-fix" >> "$SELECTED_DIRECTORY/logs/wineprefixes.log"
    elif [[ "$SELECTED_OPTION" == "--proton" ]]; then
        echo "$PROTON_VERSION" >> "$SELECTED_DIRECTORY/logs/wineprefixes.log"
    else
        echo "Wine" >> "$SELECTED_DIRECTORY/logs/wineprefixes.log"
    fi
    echo "$SELECTED_DIRECTORY" >> "$FUSION_DESKTOP_DIRECTORY/installs.log"
}

##############################################################################################################################################################################
# ACTIVATE THE WINDOW NOT RESPONDING DIALOG:                                                                                                                                 #
##############################################################################################################################################################################

reset_window_not_responding_dialog() {
    # Check if desktop environment is GNOME
    if [ "$XDG_CURRENT_DESKTOP" = "GNOME" ]; then
        # Reset the "Window not responding" Dialog in GNOME
        echo -e "$(gettext "${GREEN}The 'Window not responding' Dialog in GNOME will be reset!")${NOCOLOR}"
        gsettings reset org.gnome.mutter check-alive-timeout
    fi
}

##############################################################################################################################################################################
# RUN AUTODESK FUSION:                                                                                                                                                       #
##############################################################################################################################################################################

run_wine_autodesk_fusion() {
    # Execute the Autodesk Fusion 360
    echo -e "$(gettext "${GREEN}Starting Autodesk Fusion 360 ...${NOCOLOR}")"
    sleep 2
    source "$SELECTED_DIRECTORY/bin/autodesk_fusion_launcher.sh"
}

##############################################################################################################################################################################

check_required_packages
#download_translations
check_option "$SELECTED_OPTION"
