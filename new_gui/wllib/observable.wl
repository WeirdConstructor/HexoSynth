!@wlambda;
!@import std;

!Observable = ${
    observable_init = {
        $data._callbacks = ${};
    },
    emit = {!args = @;
        !ev_cbs = $data._callbacks.(args.0);
        !new_cbs = $[];
        $data._callbacks.(args.0) = new_cbs;

        iter cb ev_cbs {
            cb[[args]];
            std:push new_cbs cb;
        };
    },
    listen = {!(event, cb) = @;
        if is_none[$data._callbacks.(event)] {
            $data._callbacks.(event) = $[];
        };

        std:push $data._callbacks.(event) cb;
    },
};

!@export Observable = Observable;
