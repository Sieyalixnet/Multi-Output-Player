import { writeTextFile, BaseDirectory, writeFile } from '@tauri-apps/plugin-fs';
import { ElMessage } from 'element-plus';

export async function downloadText(content: string, name: string, format: string) {
        var blob = new Blob([content], { type: "text/plain;charset=utf-8" });
        const contents = await blob.arrayBuffer();
        const filename = `${name}_${String(Date.now())}.${format}`
        writeFile(filename, new Uint8Array(contents) , { baseDir: BaseDirectory.Download }).then(() => { ElMessage({ type: "success", message: `saved to: downloads/${filename}` }) }).catch((e) => { ElMessage.error(`Save error:${e}`) })
}

export function save_cookies(name: string, obj: { [key: string]: any }, except_keys?: Array<string>) {
    let new_obj: { [key: string]: any } = {}
    for (let key of Object.keys(obj)) {
        if (Array.isArray(except_keys) && except_keys.includes(key)) continue
        new_obj[key] = obj[key]
    }
    localStorage.setItem(name, JSON.stringify(new_obj))
}


export function get_cookies(name: string, obj: { [key: string]: any }, except_keys?: Array<string>) {
    try {
        let c = localStorage.getItem(name)
        if (!c) return false;
        let obj_get = JSON.parse(c)
        for (let key of Object.keys(obj)) {
            if (Array.isArray(except_keys) && except_keys.includes(key)) continue;
            obj[key] = obj_get[key]
        }
        return true
    }
    catch (e) { console.log(e) }
    return false
}