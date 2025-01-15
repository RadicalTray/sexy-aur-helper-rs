# TODO
## Doing

## Backlog
    - check install conflicts

## Interesting
    - could've used ref count ptr on some clone_path variables
    - (config): use PKGBUILD in .config/saur
        - detect and notify if upstream is updated and local PKGBUILD is not
        - could try git cloning aur repo into .config/saur/clone/<AUR REPO> and
        use that repo git commit to see if updated
        - OR use .config/saur/clone/<AUR PKG>/PKGBUILD
            - plant a .config/saur/clone/<AUR PKG>/NOTUPDATED file to check if PKGBUILD is up to date
            (remove file to tell that it's up to date)
            - refuse to upgrade if not updated
        - OR a diff like paru does it i think?

## Maybe?
    - more multithreading?
    - print stat?
    - (clear cache):
        - paru doesn't do this, i think (my paru might be broken)
            - rm dir that is not AUR package?
            - only remove outdated packages?

## Out of scope
    - [Need crate: aur_depends] automatically fetch deps
        - filter Official/AUR
            - Official
                - show "Official: {name}, ..."
            - AUR
                - show "AUR: {name}, ..."
                - [PROBLEM] how tf do i do build deps if deps need deps
        - install official
        - build aur
