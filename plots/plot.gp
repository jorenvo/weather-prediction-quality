set macros
png="set terminal png size 1800,1800 crop enhanced font \"/usr/share/fonts/truetype/times.ttf,30\" dashlength 2; set termoption linewidth 3"
eps="set terminal postscript fontfile \"/usr/share/fonts/truetype/times.ttf\"; set termoption linewidth 3;

set style line 1 linecolor rgb "#de181f" linetype 1  # Red
set style line 2 linecolor rgb "#0060ae" linetype 1  # Blue
set style line 3 linecolor rgb "#228C22" linetype 1  # Forest green

set style line 4 linecolor rgb "#18ded7" linetype 1  # opposite Red
set style line 5 linecolor rgb "#ae4e00" linetype 1  # opposite Blue
set style line 6 linecolor rgb "#8c228c" linetype 1  # opposite Forest green

# set term svg enhanced mouse size 600,400
set multiplot layout 1,2
set xdata time
set timefmt "%Y-%m-%d" # format in data.dat
set format x "%m-%d" # xtics format
# set xtics rotate by 45 right center offset 0,-2
set yrange [0:10]
set xlabel "Prediction date"
set ylabel "Temperature (°F)"

set title "Prediction 06-07"
plot "data.dat" using 1:2 title "my title" with linespoints
set title "Prediction 06-08"
plot "data.dat" using 1:3 title "my other title" with linespoints
pause -1 "Hit return to continue..."