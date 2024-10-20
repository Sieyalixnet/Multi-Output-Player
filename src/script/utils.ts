export const format_duration = (t: number) => {
    let sec = t / 1000;
    let min = Math.floor(sec / 60);
    let sec_left = sec % 60;
    return `${min}:${sec_left.toFixed(2)}`
}