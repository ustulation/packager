# After-run
## To install
sudo dpkg -i safe-launcher-v0.7.1-linux-x64_1.0_amd64.deb
## Package info
dpkg --info safe_launcher-v0.7.1-linux-x64_1.0_amd64.deb -> will give package to help uninstall -> same as -n option above
## To uninstall
sudo dpkg -r <package-name-from-above-info>
sudo rm /usr/bin/safe_launcher -> will remove symlink to `/opt/maidsafe/...`
