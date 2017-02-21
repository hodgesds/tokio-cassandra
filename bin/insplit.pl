#!/usr/bin/perl
undef $/;
$_ = <>;
$n = 0;
for $content (split(/(?=\x03\x00)/)) {
        open(OUT, ">insplit" . ++$n . ".txt");
        print OUT $content;
        close(OUT);
}
