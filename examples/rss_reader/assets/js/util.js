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
