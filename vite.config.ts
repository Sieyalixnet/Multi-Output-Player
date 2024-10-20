import { defineConfig } from 'vite'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { ElementPlusResolver } from 'unplugin-vue-components/resolvers'
import vue from '@vitejs/plugin-vue'

export default defineConfig(({ command }) => {
  let res = {
    build:{
      outDir:"./dist"
    },
    define:{
"__DEV__": command === 'serve'?true:false,

    },
    plugins: [vue(),
    AutoImport({
      resolvers: [ElementPlusResolver()],
    }),
    Components({
      resolvers: [ElementPlusResolver()],
    })
    ],
    clearScreen: false,
    server: {
      port: 14420,
      strictPort: true,
    },
    envPrefix: ["VITE_", "TAURI_"],
  };
  res.build.outDir= "./Build_Tauri"
  return res

})

