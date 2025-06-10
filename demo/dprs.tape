#!/usr/bin/env -S VHS  

Output "demo/dprs.gif"

Set FontSize 18
Set Width 1200
Set Height 800
Set TypingSpeed 100ms
Set WindowBar Colorful
Set Margin 1

Hide 
Sleep 1s 

Type "export PS1='$ '" Sleep 50ms Enter
Type "clear" Sleep 50ms Enter
Show

# DPRS Demo
Type "dprs"
Enter
Sleep 4s # Show initial UI

# Navigate containers
Down # Simulates j or ArrowDown
Sleep 1s
Down
Sleep 1s
Up # Simulates k or ArrowUp
Sleep 1s

# Copy IP
Type "c"
Sleep 2s # Allow time for toast message if any

# Stop container (no visual change in VHS, just showing command)
Type "x"
Sleep 2s

# Restart container list (as per README for dprs)
Type "r"
Sleep 2s

# Quit dprs
Type "q"
Sleep 1s
