# bi-plane vs aliens

a 3d game using opengl and sdl2. sdl2 for keyboard and because I copied a start screen from another game.

Fly bi-plane around shooting aliens, ideally before they land. Too many land that's the end of the game

Game only possible with excellent https://github.com/bwasty/learn-opengl-rs 

See a short demo here
https://youtu.be/m7fsfRz65o4 


## build
on linux to target windows
```
export PKG_CONFIG_ALLOW_CROSS=1
cargo build --target x86_64-pc-windows-gnu
```

or to switch off sound, wine should then work
```
cargo build --target x86_64-pc-windows-gnu  --features soundoff
```

or from linux
cargo build

the windows target should have al the dependencies from mingw in the code base. I've tried on a normal Windows PC
For linux maybe but I to do a package installs during dev. These maybe enough
```
sudo apt-get install libsdl2-gfx-dev libsdl2-image-dev libsdl2-ttf-dev
```

to deliver you need to zip up in the debug directory
biplanevsaliens
resources/
any dlls

```
cd target/x86_64-pc-windows-gnu/debug
zip -r biplanevsaliens_windows.zip biplanevsaliens.exe resources/* *dll
```

or
```
cd target/debug
zip -r biplanevsaliens_linux.zip biplanevsaliens resources
```

# run released code
To run built code either download and expand

https://github.com/bernardjason/biplanevsaliens/releases/download/0.1.0/biplanevsaliens_linux.zip

or

https://github.com/bernardjason/biplanevsaliens/releases/download/0.1.0/biplanevsaliens_windows.zip

and run executable biplanevsaliens or biplanevsaliens.exe
