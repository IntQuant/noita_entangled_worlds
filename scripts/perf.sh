set -e
f=$(cat $1)
a=$(echo "$f"|grep "LUA: \["|sed 's/LUA: \[//g;s/].*//;s/ //g'|awk -F',' '{col1 = col1 $1 " "; col2 = col2 $2 " "; col3 = col3 $3 " "} END {print col1; print col2; print col3}'|sed 's/ /,/g;s/,$/}/g;s/^/{/')
for b in $a;do
  echo "{mean$b,standarddeviation$b,max$b,len$b}"|kalc
done

a=$(echo "$f"|grep "LUA: {"|sed 's/LUA: {//g;s/}.*//;s/ //g'|sed 's/,/\n/g'|sort -n)
for i in $(echo "$a"|sed 's/:.*//g'|uniq);do
  b=$(echo {$(echo "$a"|grep "^$i:"|sed 's/.*://g')}|sed 's/ /,/g')
  echo -ne "$i "
  echo "{mean$b,standarddeviation$b,max$b,len$b}"|kalc
done