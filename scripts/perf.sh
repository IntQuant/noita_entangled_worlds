set -e
f=$(cat $1)
a=$(echo "$f"|grep "LUA: \["|sed 's/LUA: \[//g;s/].*//;s/ //g'|awk -F',' '{for (i=1; i<=NF; i++) cols[i] = cols[i] $i " "} END {for (i in cols) {gsub(/ $/, "", cols[i]); gsub(/ /, ",", cols[i]); print "{" cols[i] "}"}}')
for b in $a;do
  echo "{mean$b,standarddeviation$b,max$b,percentile($b,99),percentile($b,99.9),len$b}"|kalc
done

a=$(echo "$f"|grep "LUA: {"|sed 's/LUA: {//g;s/}.*//;s/ //g'|sed 's/,/\n/g'|sort -n)
for i in $(echo "$a"|sed 's/:.*//g'|uniq);do
  b=$(echo {$(echo "$a"|grep "^$i:"|sed 's/.*://g')}|sed 's/ /,/g')
  echo -ne "$i "
  echo "{mean$b,standarddeviation$b,max$b,percentile($b,99),percentile($b,99.9),len$b}"|kalc
done
