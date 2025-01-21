This will be useful in the future, once a gui is also created.

1. Setup [VBCable](https://vb-audio.com/Cable/index.htm). I recommend [this tutorial](https://www.youtube.com/watch?v=GC1aLL7cPY4).
2. Just as the tutorial shows how to change the voice in discord, you can do the same in steam. In steam this can be done by opening up a game (like tf2), shift+tab, Settings, In-Game Voice, and selecting the CABLE Output for the mic. 
3. Find your tf2 autoexec.cfg file (should be under `C:\Program Files (x86)\Steam\steamapps\common\Team Fortress 2\tf\cfg`) and add the following line to it:
```
con_logfile tf2consoleoutput.log
```
4. Go to `C:\Users\{YOUR USER HERE}\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup` and create a `CleanupTF2Log.bat` file, add the following content to it:
```bat
@echo off
set logfile="C:\Program Files (x86)\Steam\steamapps\common\Team Fortress 2\tf\tf2consoleoutput.log"
if exist %logfile% (echo. > %logfile%)
```
This cleans up the log file on each startup, so that it doesn't become too large one day.
5. Make sure you have rust installed.
6. Clone the repo
7. Input your steam ID
8. Steps above will prob be changed, this is a WIP