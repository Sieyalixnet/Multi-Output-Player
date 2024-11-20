import { reactive, watch } from "vue";

export type DetectedFile = {
    path: string,
    name:string,
    duration: number,
    _duration:number,
    sample_rate: number,
    format: string,
}

export const Detected_Files = reactive<{ data: DetectedFile[] }>({
    data: [],
})

// watch(Detected_Files, (newvalue, oldvalue) => {
//     console.log("from watch", newvalue, oldvalue)
// })

export class Device {
    name: string
    selected: boolean
    volumn: number
    constructor(name: string, selected: boolean, volumn: number) {
        this.name = name
        this.selected = selected
        this.volumn = volumn
    }

    get_playpayload() {
        return {
            name: this.name,
            volumn: this.volumn
        }
    }

}

export const Devices = reactive<{ data: Device[] }>({
    data: [],
})

export const playing_status = reactive({
    playing: false,
    path: "",
    format: "",
    total_duration:0,
    paused:false,
    duration:0,
    index:-1
})
