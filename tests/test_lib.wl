!@wlambda;
!@import std;
!@import hx;

!@export mouse_click = {
    hx:mouse_down _;
    hx:mouse_up _;
};

!@export key = {
    iter key @ {
        hx:key_down key;
        hx:key_up   key;
    };
};

