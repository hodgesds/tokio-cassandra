#!/usr/bin/perl
undef $/;
$_ = <>;
$n = 0;
for $content (split(/(?=\x83\x00)/)) {
        open(OUT, ">outsplit" . ++$n . ".txt");
        print OUT $content;
        close(OUT);
}
