import { createApp, provide } from 'vue'
// import { router } from './router'
import App from "./App.vue"
import './tailwind.css'
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import { listen,TauriEvent } from '@tauri-apps/api/event'

import { DetectedFile, Detected_Files, playing_status } from './script/rc'
import { fileDropEvent } from './types'
import { invoke } from "@tauri-apps/api/core";
import { format_duration } from './script/utils'
const app = createApp(App)
//@ts-ignore
if (__DEV__ === false) {
    document.oncontextmenu = function () { return false; }
}

listen(TauriEvent.DRAG_DROP, async (e: fileDropEvent) => {
    for (const path of e.payload.paths) {
        const res = await invoke("detect_file", { path: path }) as string
        if (res === "unknown") continue
        const r = JSON.parse(res);
        r._duration = format_duration( r.duration);
        Detected_Files.data.push(r as DetectedFile);
    }
})


// app.use(router)
app.mount('#app')