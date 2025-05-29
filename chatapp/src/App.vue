<template>
  <div class="h-full flex flex-col overflow-hidden">
    <router-view />
    <StagewiseToolbar v-if="isDev" :config="stagewiseConfig" />
  </div>
</template>

<script>
import { StagewiseToolbar } from '@stagewise/toolbar-vue';

export default {
  name: 'App',
  components: {
    StagewiseToolbar
  },
  data() {
    return {
      isDev: process.env.NODE_ENV === 'development',
      stagewiseConfig: {
        plugins: []
      }
    };
  }
};
</script>
