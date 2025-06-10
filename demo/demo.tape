Output "demo/demo.gif"

Set Shell bash
Set FontSize 16
Set Width 1200
Set Height 800
Set TypingSpeed 100ms

Hide Sleep 1s Show

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

# Open browser (no visual in VHS, just showing command)
Type "l"
Sleep 2s

# Stop container (no visual change in VHS, just showing command)
Type "x"
Sleep 2s

# Refresh container list (as per README for dprs)
Type "r"
Sleep 2s

# Quit dprs
Type "q"
Sleep 1s

Sleep 1s

# DPLW Demo
Type "dplw"
Enter
Sleep 4s # Show initial UI

# Switch between containers
Right
Sleep 1s
Right
Sleep 1s
Left
Sleep 1s

# Scroll through logs
Down
Sleep 0.5s
Down
Sleep 0.5s
Up
Sleep 0.5s
Up
Sleep 0.5s

# Refresh container list
Type "r"
Sleep 2s

# Quit dplw
Type "q"
Sleep 1s
Ctrl+L

Hide
Sleep 1s
Show
Type "echo 'DPRS Demo Complete!'"
Enter
Sleep 2s
