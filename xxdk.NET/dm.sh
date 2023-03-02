#!/bin/bash

rm -fr client1 client2 client1.log client2.log

echo "Running clients to initialize and register 2 clients with mainnet and send ourselves a message, will take 10-20s..."
date
dotnet run --ndf mainnet-ndf.json --state-dir client1 --wait 0 --message "I am client 1" > client1.log &
dotnet run --ndf mainnet-ndf.json --state-dir client2 --wait 0 --message "I am client 2" > client2.log

grep "DMRe" *.log

date

CLIENT1KEY=$(grep DMPUBKEY client1.log | head -1 | awk '{print $2}')
CLIENT1TOKEN=$(grep DMTOKEN client1.log | head -1 | awk '{print $2}')

CLIENT2KEY=$(grep DMPUBKEY client2.log | head -1 | awk '{print $2}')
CLIENT2TOKEN=$(grep DMTOKEN client2.log | head -1 | awk '{print $2}')


echo "Clients Initialized:"
echo -e "\tClient 1: $CLIENT1KEY, $CLIENT1TOKEN"
echo -e "\tClient 2: $CLIENT2KEY, $CLIENT2TOKEN"

echo "NOTE: Remember DMs send to yourself as well as target and there's no msg storage here, so duplicates are expected..."
date

dotnet run --ndf mainnet-ndf.json --state-dir client1 --message "Hi Client 2" --partner-key $CLIENT2KEY --partner-token $CLIENT2TOKEN 2>&1 >> client1.log &

dotnet run --ndf mainnet-ndf.json --state-dir client2 --message "Hi Client 1" --partner-key $CLIENT1KEY --partner-token $CLIENT1TOKEN 2>&1 >> client2.log &


echo "CTRL+C to get out of this...You should see msg receive output from 2 clients in about 5-10s"
tail -f *.log | grep "DMRe"
