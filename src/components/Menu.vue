<template>
    <el-menu :default-active="activeIndex" mode="horizontal" @select="handleSelect">
        <el-sub-menu index="2">
      <template #title>File</template>
      <el-menu-item index="loadPlaylist">Load playlist</el-menu-item>
      <el-menu-item index="savePlaylist">Save playlist</el-menu-item>
      <el-menu-item index="checkPlaylist">Check playlist</el-menu-item>
      <el-menu-item index="exit">Exit</el-menu-item>
    </el-sub-menu>

    </el-menu>
</template>

<script setup lang="ts">
import {ref} from 'vue';
import { exit } from '@tauri-apps/plugin-process';
const emit =  defineEmits<{
  (e: 'savePlaylist'): void,
  (e:'checkPlaylist'):void,
  (e:'uploadPlaylist',s:string):void
}>()

const activeIndex = ref('1');
const handleSelect = (key: string, keyPath:string) => {
  switch (key) {
    case "loadPlaylist":
            input()
            break;
        case "savePlaylist":
            emit('savePlaylist')
            break;
        case "checkPlaylist":
            emit('checkPlaylist')
            break;
        case "exit":
            exit(0)
            break;
        default:
            break;
    }
}

let input = () => {
    var __temp_InputElement__ = document.createElement("input");
    __temp_InputElement__.addEventListener("change", HandleFileChange, false);
    __temp_InputElement__.type = "file";
    (__temp_InputElement__.accept = ".mspl"),
        __temp_InputElement__.click();
};

function HandleFileChange(e:any) {
    let file = e.target.files[0];
    let fileReader = new FileReader();
    try {
        fileReader.readAsText(file, "utf-8");
        fileReader.onload = function () {
            parse(this.result);

        };
    } catch (err:any) {
    }
}


function parse(result:any) {
  emit('uploadPlaylist',result)
}
</script>

<style scoped>
.el-menu--horizontal {
  --el-menu-horizontal-height: 30px;
}
</style>