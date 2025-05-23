<template>
  <div class="flex-1 overflow-y-auto p-5 mb-10" ref="messageContainer">
    <div v-if="messages.length === 0" class="text-center text-gray-400 mt-5">
      No messages in this channel yet.
    </div>
    <div v-else>
      <div v-for="message in messages" :key="message.id" class="flex items-start mb-5">
        <img :src="`https://ui-avatars.com/api/?name=${getSender(message.sender_id).fullname.replace(' ', '+')}`" class="w-10 h-10 rounded-full mr-3" alt="Avatar" />
        <div class="max-w-4/5">
          <div class="flex items-center mb-1">
            <span class="font-bold mr-2">{{ getSender(message.sender_id).fullname }}</span>
            <span class="text-xs text-gray-500">{{ message.formattedCreatedAt }}</span>
          </div>
          <div class="text-sm leading-relaxed break-words whitespace-pre-wrap">{{ getMessageContent(message) }}</div>
          <div v-if="message.files && message.files.length > 0" class="grid grid-cols-3 gap-2 mt-2">
            <div v-for="(file, index) in message.files" :key="index" class="relative">
              <img
                :src="getFileUrl(file)"
                :class="{'h-32 object-cover cursor-pointer': true, 'w-auto h-auto': enlargedImage[message.id]}"
                @click="toggleImage(message.id)"
                alt="Uploaded file"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { getUrlBase } from '../utils';

export default {
  data() {
    return {
      enlargedImage: {},
    };
  },
  computed: {
    messages() {
      const msgs = this.$store.getters.getMessagesForActiveChannel;
      // console.log('MessageList.vue computed messages:', msgs);
      return msgs;
    },
    activeChannelId() {
      let channel = this.$store.state.activeChannel;
      if (!channel) {
        return null;
      }
      return channel.id;
    }
  },
  watch: {
    messages: {
      handler() {
        // alert("messages:" + JSON.stringify(this.messages));
        this.$nextTick(() => {
          this.scrollToBottom();
        });
      },
      deep: true
    },
    activeChannelId(newChannelId) {
      if (newChannelId) {
        this.fetchMessages(newChannelId);
      }
    }
  },
  methods: {
    fetchMessages(channelId) {
      this.$store.dispatch('fetchMessagesForChannel', channelId);
    },
    getSender(userId) {

      return this.$store.getters.getUserById(userId);
    },
    scrollToBottom() {
      const container = this.$refs.messageContainer;
      if (container) {
        container.scrollTop = container.scrollHeight;
      }
    },
    getFileUrl(file) {
      return `${getUrlBase()}${file}?token=${this.$store.state.token}`;
    },
    toggleImage(messageId) {
      this.enlargedImage[messageId] = !this.enlargedImage[messageId];
      this.enlargedImage = { ...this.enlargedImage };
    },
    getMessageContent(message) {
      // TODO: handle case where user is not logged in
      if (!this.$store.state.user) {
        return '';
      }
      // alert(message.modified_content)
      if (message.sender_id === this.$store.state.user.id) {
        return message.content;
      } else {
        return message.modified_content && message.modified_content.trim() !== ''
          ? message.modified_content
          : message.content;
      }
    }
  },
  mounted() {
    if (this.activeChannelId) {
      this.fetchMessages(this.activeChannelId);
    }
    this.scrollToBottom();
  },
  updated() {
    this.scrollToBottom();
  }
};
</script>
