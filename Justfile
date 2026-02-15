list:
    @just --list

mac-app:
    cargo bundle --release

dmg: mac-app
    uvx dmgbuild -s packaging/dmgsettings.py "Crazyflie Mini Client" "Crazyflie Mini Client.dmg"

clean:
    cargo clean
    rm -rf AppDir
    rm -f *.AppImage *.dmg miniclient.png wix/app.ico