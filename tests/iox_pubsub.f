
"before pub" .cr

"Service/Instance/Event" "hello from forth" iox.pub

"after pub" .cr

"Service/Instance/Event" iox.sub
"after sub" .cr

1000 ms ( sleep 1s to allow delivery )
"Service/Instance/Event" iox.sub@
"received:" swap . .cr
