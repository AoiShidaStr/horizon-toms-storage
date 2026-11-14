#!/bin/bash
# Headless smoke test of Tom's Simple Storage on a scratch dedicated server
# (127.0.0.1 only, offline mode, flat world, its own port), with one
# horizon-bots client that right-clicks the terminal and then clicks a slot:
#
#   run A: click the terminal's first line -> a stack must reach the bot.
#   run B: shift-click the bot's own inventory slot -> the stack must reach
#          the chest.
#
# Never touches /opt/minecraft or other mods' scratch folders.
S=/opt/build/toms-smoke
SRV=$S/server
LOG=$S/engine.log
IN=$S/console.in
ENGINE=${ENGINE:-/opt/build/target/debug/horizon-client}
BOTS=${BOTS:-/opt/build/target/debug/horizon-bots}
PORT=25597
HMOD=${HMOD:-/opt/build/mods/toms_storage/target/toms_storage.hmod}

pkill -f "[t]oms-smoke/resources" 2>/dev/null
pkill -f "[h]orizon-bots --server 127.0.0.1:$PORT" 2>/dev/null
sleep 1
rm -rf $SRV; mkdir -p $SRV/horizon-mods
cp "$HMOD" $SRV/horizon-mods/
echo "mod.toms_storage:true" > $SRV/horizon-mods.txt
echo "eula=true" > $SRV/eula.txt
cat > $SRV/server.properties <<P
server-ip=127.0.0.1
server-port=$PORT
online-mode=false
level-type=minecraft\\:flat
gamemode=survival
force-gamemode=true
difficulty=normal
spawn-protection=0
pause-when-empty-seconds=0
view-distance=4
simulation-distance=4
generate-structures=false
spawn-monsters=false
spawn-animals=false
white-list=false
enforce-whitelist=false
max-players=8
P
: > $IN
cmd() { echo "[smoke] > $*" >> $LOG; echo "$*" >> $IN; }

cd $SRV
export HORIZON_NO_AUDIO=1 TOMS_TRACE=1
(tail -f $IN | $ENGINE --resources $S/resources --dedicated --nogui > $LOG 2>&1; echo "[smoke] engine exited $?" >> $LOG) &
for i in $(seq 1 240); do grep -q "Done (" $LOG && break; grep -q "engine exited" $LOG && { echo "server did not start"; exit 1; }; sleep 1; done
echo "[smoke] server up after ${i}s" >> $LOG

# Beside the terminal: a connector, then a chest holding stone and diamonds.
cmd "gamerule respawn_radius 0"
cmd "setblock 3 -60 0 toms_storage:storage_terminal"
cmd "setblock 4 -60 0 toms_storage:inventory_connector"
cmd "setblock 5 -60 0 minecraft:chest"
cmd "setblock 3 -59 0 toms_storage:inventory_cable"
cmd "setblock 3 -90 0 minecraft:air"
sleep 1
cmd "item replace block 5 -60 0 container.0 with minecraft:stone 64"
cmd "item replace block 5 -60 0 container.1 with minecraft:diamond 7"
cmd "data get block 5 -60 0 Items"
sleep 1

echo "[smoke] ===== run A: right-click the terminal, click line 0" >> $LOG
$BOTS --server 127.0.0.1:$PORT --bots 1 --behaviour idle --height 0 --ground --seconds 14 \
  --name Take --click 3,-60,0:0:0:0 --report $S/bots-a.json > $S/bots-a.log 2>&1 &
for i in $(seq 1 80); do grep -q "Take0 joined" $LOG && break; sleep 0.25; done
sleep 10
cmd "data get entity Take0 Inventory"
cmd "execute if entity @e[type=minecraft:item]"
cmd "data get block 5 -60 0 Items"
sleep 4

echo "[smoke] ===== run B: shift-click a stack of gold into the network" >> $LOG
$BOTS --server 127.0.0.1:$PORT --bots 1 --behaviour idle --height 0 --ground --seconds 16 \
  --name Store --click 3,-60,0:54:0:1 --report $S/bots-b.json > $S/bots-b.log 2>&1 &
for i in $(seq 1 80); do grep -q "Store0 joined" $LOG && break; sleep 0.25; done
# The bot clicks 4 s after joining: fill its first main-inventory slot now.
cmd "item replace entity Store0 container.9 with minecraft:gold_ingot 32"
sleep 6
cmd "data get entity Store0 Inventory"
cmd "data get block 5 -60 0 Items"
sleep 10

echo "[smoke] ===== run C: only an open crate in the network" >> $LOG
# The crate takes the chest's place, so the terminal's only container is the
# new block; a click must reach the emerald inside it.
cmd "kill @e[type=minecraft:item]"
cmd "setblock 5 -60 0 toms_storage:open_crate"
cmd "setblock 6 -60 0 toms_storage:crafting_terminal"
sleep 1
cmd "item replace block 5 -60 0 container.0 with minecraft:emerald 5"
cmd "data get block 5 -60 0 Items"
sleep 1
$BOTS --server 127.0.0.1:$PORT --bots 1 --behaviour idle --height 0 --ground --seconds 14 \
  --name Crate --click 3,-60,0:0:0:0 --report $S/bots-c.json > $S/bots-c.log 2>&1 &
for i in $(seq 1 80); do grep -q "Crate0 joined" $LOG && break; sleep 0.25; done
sleep 10
cmd "data get entity Crate0 Inventory"
cmd "data get block 5 -60 0 Items"
sleep 4

cmd "list"
cmd "stop"
for i in $(seq 1 60); do grep -q "engine exited" $LOG && break; sleep 1; done
pkill -f "[h]orizon-bots --server 127.0.0.1:$PORT"
pkill -f "[t]oms-smoke/resources"
pkill -f "[t]ail -f $IN"
echo "[smoke] DONE" >> $LOG
echo "--- trace"
grep "toms_storage:" $LOG || true
