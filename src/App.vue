<template>
    <div style="width: 100vw;height: 100vh;">
        <Menu @savePlaylist="savePlaylist" @uploadPlaylist="uploadPlaylist" @checkPlaylist="checkPlaylist"></Menu>
        <div class=" flex flex-row justify-between items-start w-[100%] h-[85%] border-b  border-slate-200">


            <div class=" w-[70%] h-[100%] overflow-y-scroll border-r  border-slate-200">

                <el-table :data="Detected_Files.data" v-if="Detected_Files.data.length > 0"
                    :row-class-name="tableRowClassName">
                    <el-table-column fixed prop="name" label="Path" width="360" show-overflow-tooltip/>
                    <el-table-column prop="_duration" label="Duration" width="120" />
                    <el-table-column prop="sample_rate" label="Sample Rate" width="120" />
                    <el-table-column prop="format" label="Format" width="120" />
                    <el-table-column fixed="right" label="Operations" min-width="120">
                        <template #default="scope">
                            <el-button circle type="primary" size="small" @click="play(scope.$index)">
                                <PlayOne theme="outline" size="22" fill="#333" />
                            </el-button>
                            <el-button circle type="danger" size="small" @click="remove(scope.$index)">
                                <Delete theme="outline" size="18" fill="#333" />
                            </el-button>

                        </template>
                    </el-table-column>
                </el-table>
                <el-empty description="Drag & Drop Audio Files here. 请把音乐文件拖到这里" v-else />
            </div>
            <div class="flex-1 flex flex-col justify-start items-start mx-10">

                <div v-for="item in Devices.data" class="w-[100%]">
                    <el-checkbox :key="item.name" v-model="item.selected" :label="item.name" size="large"
                        :disabled="playing_status.playing === true" />
                    <el-slider v-if="item.selected" v-model="item.volumn" :step="0.01" :max="1" :min="0"
                        @change="volumn_change(item.name, item.volumn)" />
                </div>
                <div class=" w-[100%] flex flex-row justify-end items-center">
                    <el-button class=" mx-4" size="large" type="primary" @click="refresh"
                        :disabled="playing_status.playing === true">Refresh Devices</el-button>
                </div>
            </div>
        </div>
        <div class="w-[100%] flex flex-row justify-center items-center my-4 ">
            <div class="w-[60%] flex flex-col justify-start items-center">
                <div class="w-[100%] flex flex-row justify-between items-center">
                    <div class="w-[70%] mx-8">

                        <el-slider v-model="playing_status.duration" :step="100" :max="playing_status.total_duration"
                            :min="0" @input="goto($event)"
                            :disabled="playing_status.playing === false || ['MP3', 'WAV'].includes(playing_status.format.toUpperCase()) === false"
                            :format-tooltip="format_duration" />

                    </div>
                    <div class="flex-1 flex flex-row justify-end items-center">
                        <p class="mr-4">{{ format_duration(playing_status.duration) }}</p>
                        <el-button class="" type="danger" @click="stop" :disabled="playing_status.playing === false"
                            circle>
                            <Square theme="outline" size="24" fill="#333" />
                        </el-button>
                        <el-button class="" type="primary" @click="pause" :disabled="playing_status.playing === false"
                            v-if="playing_status.paused === false" circle>
                            <Pause theme="outline" size="24" fill="#333" />
                        </el-button>
                        <el-button class="" type="primary" @click="resume" :disabled="playing_status.playing === false"
                            v-if="playing_status.paused === true" circle>
                            <PlayOne theme="outline" size="24" fill="#333" />
                        </el-button>
                    </div>
                </div>
                <el-alert type="error" title="NOT SUPPORT SEEK. 不支持跳转的格式."
                    v-if="playing_status.playing === true && ['MP3', 'WAV'].includes(playing_status.format.toUpperCase()) === false"
                    :closable="false" />
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import { Detected_Files, Devices, Device, playing_status, DetectedFile } from "./script/rc";
import { invoke } from "@tauri-apps/api/core";
import { Square, Pause, PlayOne, Delete } from '@icon-park/vue-next'
import { ElMessage } from 'element-plus'
import { format_duration } from "./script/utils"
import { downloadText } from "./script/save";
import Menu from "./components/Menu.vue";

onMounted(() => {
    refresh()
    refresh_playing()
})
const savePlaylist = ()=>{
    if(Detected_Files.data.length === 0){
        ElMessage({
            message: "Playlist is empty. 播放列表为空",
            type: "error"
        })
        return
    }
   downloadText(JSON.stringify(Detected_Files.data), "playlist", "mspl")
}
const uploadPlaylist=(text:string)=>{
    const data = JSON.parse(text)
    Detected_Files.data.splice(0)
    for (let i = 0; i < data.length; i++) {
        const item = data[i];
        Detected_Files.data.push(item)
    }
}
const checkPlaylist=async ()=>{
    const result:DetectedFile[] = []
    for (let i = 0; i < Detected_Files.data.length; i++) {
        const path = Detected_Files.data[i].path
        const res = await invoke("detect_file", { path: path }) as string
        if (res === "unknown") continue
        const r = JSON.parse(res);
        r._duration = format_duration( r.duration);
        result.push(r)
    }
    Detected_Files.data.splice(0)
    Detected_Files.data.push(...result)
}

const _refresh_playing = async () => {
    const res = await invoke("get_pos") as any;
    if (res === "NONE") {
        playing_status.playing = false
        playing_status.path = ""
        playing_status.duration = 0
    } else {

        let r: { duration: number, is_paused: boolean } = JSON.parse(res);
        playing_status.duration = r.duration;
        playing_status.paused = r.is_paused;
        if (playing_status.total_duration - playing_status.duration <= 200) {
            let recent_index = playing_status.index
            await stop()
            setTimeout(async () => {
                if (recent_index < Detected_Files.data.length - 1) {
                    console.log("play",recent_index+ 1)
                    await play(recent_index + 1)
                } else{
                    await play(0)
                }
            }, 200)
        }
    }

}
const refresh_playing = () => {
    setTimeout(async () => {
        await _refresh_playing()
        refresh_playing()
    }, 200)
}
const tableRowClassName = ({ row, rowIndex }: { row: any, rowIndex: number }) => {
    if (rowIndex === playing_status.index) {
        return "playing"
    }
}
const volumn_change = async (name: string, volumn: number) => {
    await invoke("set_volumn", { name: name, volumn: volumn })
}
const refresh = async () => {
    if (playing_status.playing === true) return
    const res = await invoke("refresh_devices") as string
    const data: { devices: Array<string> } = JSON.parse(res)
    const selected: Array<ReturnType<Device["get_playpayload"]>> = []
    for (let item of Devices.data) {
        if (item.selected) {
            selected.push(item.get_playpayload())
        }
    }
    Devices.data.splice(0)
    for (let i = 0; i < data.devices.length; i++) {
        const item = data.devices[i]
        let finished = false
        for (const selected_item of selected) {
            if (selected_item.name === item) {
                Devices.data.push(new Device(item, true, selected_item.volumn))
                finished = true
                break;
            }
        }
        if (finished === false) Devices.data.push(new Device(item, false, 1))
    }
}

const play = async (index: number) => {
    await stop()
    const selected_devices = Devices.data.filter(item => item.selected).map(item => item.get_playpayload())
    if (selected_devices.length === 0) {
        ElMessage.error('Please select output devices. 请选择输出设备.')
        return
    }
    await invoke("play", {
        path: Detected_Files.data[index].path, outputDevices: selected_devices
    })
    playing_status.playing = true
    playing_status.path = Detected_Files.data[index].path
    playing_status.total_duration = Detected_Files.data[index].duration
    playing_status.format = Detected_Files.data[index].format
    playing_status.index = index

}

const remove = (index: number) => {
    Detected_Files.data.splice(index, 1)
}
const stop = async () => {
    await invoke("stop")
    playing_status.playing = false
    playing_status.path = ""
    playing_status.index = -1
}
const pause = () => {
    invoke("pause")
}
const resume = () => {
    invoke("resume")
}
const goto = (event: number) => {
    invoke("goto", { ms: event })
}


</script>

<style>
.el-table .playing {
    --el-table-tr-bg-color: rgba(232, 85, 85, 0.2);
}
</style>