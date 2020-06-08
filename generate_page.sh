#!/usr/bin/env bash
set -euo pipefail
# set -o xtrace

pushd csv &> /dev/null
ALL_TEMPS=$(cut -f 2 -- *.csv | grep '[0-9]')
ALL_TEMPS+=$'\n'
ALL_TEMPS+=$(cut -f 3 -- *.csv | grep '[0-9]')
ALL_TEMPS=$(echo "${ALL_TEMPS}" | sort -n)

MIN_TEMP=$(echo "${ALL_TEMPS}" | head -n 1)
MAX_TEMP=$(echo "${ALL_TEMPS}" | tail -n 1)
MIN_TEMP=$((MIN_TEMP-1))
MAX_TEMP=$((MAX_TEMP+1))

echo "min: ${MIN_TEMP}, max: ${MAX_TEMP}"

for CSV_FILENAME in *.csv; do
    GNUPLOT_SCRIPT="
set style line 1 \
    linecolor rgb '#0060ad' \
    linetype 1 linewidth 2 \
    pointtype 7 pointsize 0.5

set style line 2 \
    linecolor rgb '#dd181f' \
    linetype 1 linewidth 2 \
    pointtype 7 pointsize 0.5

set term svg enhanced mouse size 300,200
set xdata time
set timefmt '%Y-%m-%d' # format in data.dat
set format x '%m-%d' # xtics format
set offsets 1, 1, 0, 0

set xtics rotate by 45 right offset 0,0
set yrange [${MIN_TEMP}:${MAX_TEMP}]
set xlabel 'Prediction date'
set ylabel 'Temperature (°F)'
set datafile separator '\t'

set key off

set title 'Prediction ${CSV_FILENAME%%.*}'
plot '${CSV_FILENAME}' using 1:2 title 'low' with linespoints linestyle 1,\
     '${CSV_FILENAME}' using 1:3 title 'high' with linespoints linestyle 2
"
    FILENAME="${CSV_FILENAME%%.*}.svg"
    echo "Generating ${FILENAME}..."
    echo "${GNUPLOT_SCRIPT}" | gnuplot > "${FILENAME}"
done

IMAGES=""
for SVG_FILENAME in *.svg; do
    IMAGES+="      <embed src="'"'"${SVG_FILENAME}"'"'" type="'"'"image/svg+xml"'"'"/>
"
done

HTML='<!DOCTYPE html>
<hmtl>
  <head>
    <meta charset="UTF-8">
    <title>Predictions</title>
    <style>
      .container {
        display: flex; 
        flex-wrap: wrap;
      }
    </style>
  </head>
  <body>
    <div class="container"/>
'
HTML+="${IMAGES}"
HTML+='
    </div>
  </body>
</hmtl>'

echo "${HTML}" > out.html

popd &> /dev/null
