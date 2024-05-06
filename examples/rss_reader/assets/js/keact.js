/**
 * Use native event
 * @param {*} value_fn 
 * @param {String} notify_event 
 * @returns 
 */
export function useNativeProp(value_fn, notify_event) {
    let [state, _] = useHook((compUpdateFn) => {
        let value = value_fn();
        let handler = () => {
            compUpdateFn(value_fn());
        };
        app.addListener(notify_event, handler);
        return { value, handler };
    }, (state, new_value) => {
        state.value = new_value;
        return [state, true];
    }, (state) => {
        app.removeListener(notify_event, state.handler);
    });
    return state.value;
}

/**
 * Use state
 * @param {*} init_fn 
 * @returns 
 */
export function useState(init_fn) {
    return useHook(() => {
        return (typeof init_fn === "function") ? init_fn() : init_fn;
    }, (state, new_value) => {
        return [new_value, true];
    });
}

/**
 * Use effect
 * @param {*} setup_fn
 * @param {*} deps
 * @returns 
 */
/*
export function useEffect(setup_fn, deps) {
    let [state, _] = useHook(() => {
        let cleanup = setup_fn();
        return {
            setup: setup_fn,
            deps,
            cleanup,
        };
    }, (state, new_value) => {
        return [new_value, true];
    }, (state) => {
        if (typeof state.cleanup === 'function') {
            state.cleanup();
        }        
    });
    if (deps === undefined || !Object.is(deps, state.deps)) {
        if (typeof state.cleanup === 'function') {
            state.cleanup();
        }
        let cleanup = setup_fn();
        state = {
            setup: setup_fn,
            deps,
            cleanup,
        };
    }
}
*/

class Context {
    constructor() {
        this.id = __createContextId();
        this.listeners = [];
        this.value = undefined;
    }
    get Provider() {
        let id = this.id;
        return ({ value }, kids) => {
            __provideContext(id, value);
            let l = this.listeners;
            l.forEach((cb) => cb(value));
            return kids;
        };
    }
    addListener(cb) {
        this.listeners.push(cb);
    }
    removeListener(cb) {
        this.listeners.filter((a) => a != cb);
    }
}

/**
 * Create context
 * @returns [context value, context provider]
 */
export function createContext() {
    let ctx = new Context();
    return [ctx, ctx.Provider];
}
/**
 * Use context
 * @param {*} ctx 
 * @returns context value
 */
export function useContext(ctx) {
    if (!(ctx instanceof Context)) {
        throw "useContext: param must be Context";
    }
    let [state, _] = useHook(
        (updater) => {
            ctx.addListener(updater);
            return { value: __useContext(ctx.id), updater };
        },
        (state, new_value) => {
            state.value = new_value;
            return [state, true];
        },
        (state) => {
            context.removeListener(state.updater);
        });
    return state.value;
}

function sameArray(a, b) {
    if (Array.isArray(a) && Array.isArray(b)) {
        if (a.length != b.length)
            return false;
        for (var i = 0; i < a.length; ++i) {
            if (!Object.is(a[i], b[i]))
                return false;
        }
        return true;
    } else {
        return Object.is(a, b);
    }
}
/**
 * Use effect
 * @param {*} effect_fn
 * @param {Array?} deps
 * @returns 
 */
export function useEffect(effect_fn, deps) {
    return __useEffect(effect_fn, deps, sameArray);
}

export function createRef() {
    return {
        current: null
    };
}

export function useRef() {
    return useState(() => {
        current: null
    })[0];
}
