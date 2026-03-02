<template>
  <div 
    class="w-full h-full relative flex flex-col items-center justify-center group"
    @mousemove="handleMouseMove"
    @mouseleave="handleMouseLeave"
    @contextmenu.prevent="handleContextMenu"
    ref="containerRef"
  >
    <!-- Toolbar -->
    <div 
      id="responsiveDiv"
      :class="computedToolbarClass"
      data-tauri-drag-region
    >
      <!-- File Name (Pinned Mode) -->
      <!-- <div 
        v-if="mode === 2 && !isFullScreen" 
        class="absolute left-20 text-sm text-base-content/70 truncate select-none"
        :style="{ maxWidth: filenameMaxWidth + 'px' }"
        data-tauri-drag-region
      >
        {{ fileIndex + 1 }}/{{ fileCount }} {{ file?.name }}
      </div> -->
      
      <div ref="buttonsRef" class="flex items-center space-x-1">
        <TButton
          :icon="IconPrev"
          :disabled="fileIndex <= 0 || isSlideShow"
          :tooltip="$t('image_viewer.toolbar.prev')"
          @click="triggerPrev" 
        />
        <TButton
          :icon="IconNext"
          :disabled="fileIndex < 0 || fileIndex >= fileCount - 1 || isSlideShow"
          :tooltip="$t('image_viewer.toolbar.next')"
          @click="triggerNext" 
        />
        <TButton
          :icon="isSlideShow ? IconPause : IconPlay"
          :disabled="fileIndex < 0"
          :selected="isSlideShow"
          :tooltip="(isSlideShow ? $t('image_viewer.toolbar.pause') : $t('image_viewer.toolbar.slide_show')) + ` (${getSlideShowInterval(config.settings.slideShowInterval)}s)`"
          @click="handleToggleSlideShow" 
        />
        <TButton
          :icon="IconZoomOut"
          :disabled="fileIndex < 0 || imageScale <= imageMinScale || isSlideShow"
          :tooltip="$t('image_viewer.toolbar.zoom_out') + ` (${(imageScale * 100).toFixed(0)}%)`"
          @click="zoomOut"
        />
        <TButton
          :icon="IconZoomIn"
          :disabled="fileIndex < 0 || imageScale >= imageMaxScale || isSlideShow"
          :tooltip="$t('image_viewer.toolbar.zoom_in') + ` (${(imageScale * 100).toFixed(0)}%)`"
          @click="zoomIn" 
        />
        <TButton
          :icon="!isZoomFit ? IconZoomFit : IconZoomActual"
          :disabled="fileIndex < 0 || isSlideShow"
          :tooltip="(!isZoomFit ? $t('image_viewer.toolbar.zoom_fit') : $t('image_viewer.toolbar.zoom_actual')) + ` (${(imageScale * 100).toFixed(0)}%)`"
          @click="$emit('update:isZoomFit', !isZoomFit)"
        />
        <template v-if="showExtraIcons && mode !== 2">
          <IconSeparator class="t-icon-size-sm text-base-content/30" />
          <TButton
            :icon="file?.is_favorite ? IconHeartFilled : IconHeart"
            :disabled="fileIndex < 0 || isSlideShow"
            :selected="file?.is_favorite && !isSlideShow"
            :tooltip="file?.is_favorite ? $t('menu.meta.unfavorite') : $t('menu.meta.favorite')"
            @click="$emit('item-action', { action: 'favorite', index: fileIndex })"
          />
          <ContextMenu
            :menuItems="ratingMenuItems"
            :disabled="fileIndex < 0 || isSlideShow"
            @open-change="handleMenuOpenChange"
            @click.stop
          >
            <template #trigger="{ toggle }">
              <TButton
                :icon="Number(file?.rating || 0) > 0 ? IconStarFilled : IconStar"
                :disabled="fileIndex < 0 || isSlideShow"
                :selected="Number(file?.rating || 0) > 0 && !isSlideShow"
                :tooltip="ratingButtonTooltip"
                @click.stop="toggle"
              />
            </template>
          </ContextMenu>
          <TButton
            :icon="IconTag"
            :disabled="fileIndex < 0 || isSlideShow"
            :selected="file?.has_tags && !isSlideShow"
            :tooltip="$t('menu.meta.tag')"
            @click="$emit('item-action', { action: 'tag', index: fileIndex })"
          />
          <TButton
            :icon="IconComment"
            :disabled="fileIndex < 0 || isSlideShow"
            :selected="!!file?.comments && !isSlideShow"
            :tooltip="$t('menu.meta.comment')"
            @click="$emit('item-action', { action: 'comment', index: fileIndex })"
          />
          <TButton
            :icon="IconRotate"
            :disabled="fileIndex < 0 || isSlideShow"
            :iconStyle="{ transform: `rotate(${file?.rotate ?? 0}deg)`, transition: 'transform 0.3s' }"
            :selected="file?.rotate % 360 > 0 && !isSlideShow"
            :tooltip="$t('menu.meta.rotate')"
            @click="$emit('item-action', { action: 'rotate', index: fileIndex })"
          />
        </template>
        <ContextMenu v-if="mode !== 2"
          ref="contextMenuRef"
          :iconMenu="IconMore"
          :menuItems="singleFileMenuItems"
          :disabled="fileIndex < 0 || isSlideShow"
          @open-change="handleMenuOpenChange"
          @click.stop
        />
        <IconSeparator v-if="mode !== 2" class="t-icon-size-sm text-base-content/30" />
        <TButton v-if="mode === 0"
          :icon="!isFullScreen ? IconFullScreen : IconRestoreScreen"
          :tooltip="!isFullScreen ? $t('image_viewer.toolbar.fullscreen') : $t('image_viewer.toolbar.exit_fullscreen')"
          @click="$emit('toggle-full-screen')"
        />
        <TButton v-if="mode !== 2"
          :icon="config.mediaViewer.isPinned ? IconPin : IconUnPin"
          :disabled="fileIndex < 0"
          :tooltip="!config.mediaViewer.isPinned ? $t('image_viewer.toolbar.pin') : $t('image_viewer.toolbar.unpin')"
          @click="config.mediaViewer.isPinned = !config.mediaViewer.isPinned"
        />
        <TButton
          v-if="mode === 0"
          :icon="IconClose"
          :tooltip="$t('image_viewer.toolbar.close')"
          @click.stop="$emit('close')"
        />
      </div>
    </div>

    <!-- Previous Button (Overlay) -->
    <button 
      v-if="!isSlideShow"
      class="absolute left-2 top-1/2 -translate-y-1/2 z-70 p-2 rounded-full bg-base-100/30 backdrop-blur-md transition-opacity duration-200"
      :class="[ 
        isHoverLeft ? (hasPrevious ? 'opacity-100 pointer-events-auto hover:text-base-content hover:bg-base-100/80 cursor-pointer' : 'opacity-30 cursor-default') : 'opacity-0 pointer-events-none' 
      ]"
      :disabled="!hasPrevious"
      @click.stop="triggerPrev"
      @dblclick.stop
    >
      <IconLeft class="w-8 h-8" />
    </button>

    <!-- Next Button (Overlay) -->
    <button 
      v-if="!isSlideShow"
      class="absolute right-2 top-1/2 -translate-y-1/2 z-70 p-2 rounded-full bg-base-100/30 backdrop-blur-md transition-opacity duration-200"
      :class="[ 
        isHoverRight ? (hasNext ? 'opacity-100 pointer-events-auto hover:text-base-content hover:bg-base-100/80 cursor-pointer' : 'opacity-30 cursor-default') : 'opacity-0 pointer-events-none' 
      ]"
      :disabled="!hasNext"
      @click.stop="triggerNext"
      @dblclick.stop
    >
      <IconRight class="w-8 h-8" />
    </button>

    <!-- Close Button (Top Right) -->
    <button 
      v-if="mode === 0 && !config.mediaViewer.isPinned"
      class="absolute right-2 top-2 z-90 p-2 rounded-full text-base-content/70 hover:text-base-content hover:bg-base-100/70 cursor-pointer"
      @click.stop="$emit('close')"
      @dblclick.stop
    >
      <IconClose class="w-4 h-4" />
    </button>

    <div
      v-if="mode === 0 && quickViewStatusBadges.length > 0"
      class="pointer-events-none absolute inset-x-0 top-0 z-80 h-16 bg-linear-to-b from-black/48 via-black/12 to-transparent"
    />
    <div
      v-if="mode === 0 && quickViewStatusBadges.length > 0"
      class="pointer-events-none absolute left-2 top-2 z-90 flex max-w-[calc(100%-4rem)] flex-wrap gap-1"
    >
      <div
        v-for="badge in quickViewStatusBadges"
        :key="badge.key"
        :class="['thumb-badge', badge.highlight ? 'thumb-badge-highlight' : 'thumb-badge-muted']"
      >
        <component
          v-if="badge.icon"
          :is="badge.icon"
          class="h-3.5 w-3.5 shrink-0"
        />
        <span v-if="badge.label" class="leading-none">{{ badge.label }}</span>
      </div>
    </div>

    <div class="flex-1 w-full min-h-0 relative" @dblclick="$emit('media-dblclick')">
      <Image v-if="file?.file_type === 1"
        ref="mediaRef"
        :filePath="file?.file_path" 
        :fileId="file?.id"
        :nextFilePath="nextFilePath"
        :rotate="file?.rotate ?? 0" 
        :isZoomFit="isZoomFit"
        :isSlideShow="isSlideShow"
        @update:isZoomFit="(val: boolean) => $emit('update:isZoomFit', val)"
        @scale="(e) => $emit('scale', e)"
        @viewport-change="(e) => $emit('viewport-change', e)"
        @message-from-image-viewer="handleMessageFromImageViewer"
      ></Image>
      
      <Video v-if="file?.file_type === 2"
        ref="mediaRef"
        :filePath="file?.file_path"
        :rotate="file?.rotate ?? 0"
        :isZoomFit="isZoomFit"
        :isSlideShow="isSlideShow"
        @scale="(e) => $emit('scale', e)"
        @viewport-change="(e) => $emit('viewport-change', e)"
        @message-from-video-viewer="handleMessageFromImageViewer"
        @slideshow-next="emit('slideshow-next')"
      ></Video>
    </div>

    <ToolTip ref="toolTipRef" />
  </div>
</template>

<script setup lang="ts">
import { defineAsyncComponent, ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { config, libConfig } from '@/common/config';
import { isWin, getSlideShowInterval } from '@/common/utils';

import Image from '@/components/Image.vue';
import ToolTip from '@/components/ToolTip.vue';
import TButton from '@/components/TButton.vue';
import { 
  IconLeft, 
  IconRight,
  IconPrev,
  IconNext,
  IconPlay,
  IconPause,
  IconZoomIn,
  IconZoomOut,
  IconZoomFit,
  IconZoomActual,
  IconFullScreen,
  IconRestoreScreen,
  IconPin,
  IconUnPin,
  IconSeparator,
  IconClose,
  IconMore,
  IconHeart,
  IconHeartFilled,
  IconStar,
  IconStarFilled,
  IconTag,
  IconComment,
  IconRotate,
} from '@/common/icons';
import { isMac } from '@/common/utils';
import ContextMenu from '@/components/ContextMenu.vue';
import { useFileMenuItems } from '@/common/fileMenu';

const Video = defineAsyncComponent(() => import('@/components/Video.vue'));

const props = defineProps({
  // 0: quick view, 1: filmstrip, 2: image viewer
  mode: {
    type: Number,
    default: 0
  },
  isFullScreen: {
    type: Boolean,
    default: false
  },
  file: {
    type: Object,
    default: null
  },
  hasPrevious: {
    type: Boolean,
    default: false
  },
  hasNext: {
    type: Boolean,
    default: false
  },
  fileIndex: {
    type: Number,
    default: -1
  },
  fileCount: {
    type: Number,
    default: 0
  },
  nextFilePath: {
    type: String,
    default: ''
  },
  isSlideShow: {
    type: Boolean,
    default: false
  },
  imageScale: {
    type: Number,
    default: 1
  },
  imageMinScale: {
    type: Number,
    default: 0
  },
  imageMaxScale: {
    type: Number,
    default: 10
  },
  isZoomFit: {
    type: Boolean,
    default: true
  },
});

const emit = defineEmits([
  'prev', 
  'next', 
  'toggle-slide-show', 
  'close', 
  'scale', 
  'update:isZoomFit', 
  'item-action', 
  'toggle-full-screen', 
  'slideshow-next', 
  'media-dblclick', 
  'viewport-change'
]);

const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const contextMenuRef = ref<any>(null);
const containerRef = ref<HTMLElement | null>(null);
const mediaRef = ref<any>(null);
const toolTipRef = ref<any>(null);
const isHoverLeft = ref(false);
const isHoverRight = ref(false);
const isHoverTop = ref(false);
const isHoverBottom = ref(false);
const toolbarPosition = ref<'top' | 'bottom'>('top');
const hasOpenMenu = ref(false);

// Responsive toolbar
const containerWidth = ref(0);
const buttonsRef = ref<HTMLElement | null>(null);
const buttonsWidth = ref(0);
const filenameMaxWidth = computed(() => {
  if (containerWidth.value > 0 && buttonsWidth.value > 0) {
    const val = (containerWidth.value / 2) - (buttonsWidth.value / 2) - 100;
    return Math.max(0, val);
  }
  return 200; // Fallback
});
const showExtraIcons = computed(() => containerWidth.value > 600);
const ratingButtonTooltip = computed(() => {
  const rating = Number(props.file?.rating || 0);
  return rating > 0 ? `${localeMsg.value.favorite.ratings}: ${rating}` : localeMsg.value.favorite.ratings;
});
const ratingMenuItems = computed(() => {
  const rating = Number(props.file?.rating || 0);
  return [
    {
      label: localeMsg.value.favorite.clear_rating,
      icon: IconStar,
      action: () => emit('item-action', { action: 'rating-0', index: props.fileIndex }),
    },
    { label: '-', action: null },
    {
      label: localeMsg.value.favorite.five_stars,
      icon: rating === 5 ? IconStarFilled : IconStar,
      shortcut: isMac ? '⌘5' : 'Ctrl+5',
      action: () => emit('item-action', { action: 'rating-5', index: props.fileIndex }),
    },
    {
      label: localeMsg.value.favorite.four_stars,
      icon: rating === 4 ? IconStarFilled : IconStar,
      shortcut: isMac ? '⌘4' : 'Ctrl+4',
      action: () => emit('item-action', { action: 'rating-4', index: props.fileIndex }),
    },
    {
      label: localeMsg.value.favorite.three_stars,
      icon: rating === 3 ? IconStarFilled : IconStar,
      shortcut: isMac ? '⌘3' : 'Ctrl+3',
      action: () => emit('item-action', { action: 'rating-3', index: props.fileIndex }),
    },
    {
      label: localeMsg.value.favorite.two_stars,
      icon: rating === 2 ? IconStarFilled : IconStar,
      shortcut: isMac ? '⌘2' : 'Ctrl+2',
      action: () => emit('item-action', { action: 'rating-2', index: props.fileIndex }),
    },
    {
      label: localeMsg.value.favorite.one_star,
      icon: rating === 1 ? IconStarFilled : IconStar,
      shortcut: isMac ? '⌘1' : 'Ctrl+1',
      action: () => emit('item-action', { action: 'rating-1', index: props.fileIndex }),
    },
  ];
});

const quickViewStatusBadges = computed(() => {
  const badges: Array<{ key: string; icon: any; label?: string; highlight?: boolean }> = [];
  const rating = Number(props.file?.rating || 0);

  if (props.file?.is_favorite) {
    badges.push({
      key: 'favorite',
      icon: IconHeartFilled,
      label: rating > 0 ? `${rating}` : undefined,
      highlight: true,
    });
  } else if (rating > 0) {
    badges.push({
      key: 'rating',
      icon: IconStarFilled,
      label: `${rating}`,
      highlight: true,
    });
  }

  return badges;
});
let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  resizeObserver = new ResizeObserver((entries) => {
    for (const entry of entries) {
      if (entry.target === containerRef.value) {
        containerWidth.value = entry.contentRect.width;
      } else if (entry.target === buttonsRef.value) {
        buttonsWidth.value = entry.contentRect.width;
      }
    }
  });

  if (containerRef.value) {
    resizeObserver.observe(containerRef.value);
  }
  if (buttonsRef.value) {
    resizeObserver.observe(buttonsRef.value);
  }
});

onBeforeUnmount(() => {
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
});

function handleMouseMove(e: MouseEvent) {
  if (!containerRef.value) return;
  
  const rect = containerRef.value.getBoundingClientRect();
  const x = e.clientX - rect.left;
  const y = e.clientY - rect.top;
  const width = rect.width;
  const height = rect.height;
  
  if (width > 0 && height > 0) {
    isHoverLeft.value = x < width * 0.1;
    isHoverRight.value = x > width * 0.9;
    isHoverTop.value = y < 60;
    isHoverBottom.value = y > height - 60;

    if (y < height * 0.5) {
      toolbarPosition.value = 'top';
    } else {
      toolbarPosition.value = 'bottom';
    }
  }
}

function handleMouseLeave() {
  isHoverLeft.value = false;
  isHoverRight.value = false;
  isHoverTop.value = false;
  isHoverBottom.value = false;
}

function handleContextMenu(e: MouseEvent) {
  if (contextMenuRef.value) {
    contextMenuRef.value.open(e.clientX, e.clientY);
  }
}

const computedToolbarClass = computed(() => {
  const commonClasses = 'absolute z-[80] h-10 flex flex-row items-center justify-center select-none';
  
  const isPinned = props.isFullScreen ? false : (props.mode === 2 ? true : config.mediaViewer.isPinned);

  if (isPinned) {
    // Fixed Top Bar
    return `${commonClasses} relative top-0 left-0 w-full`;
  } else {
    // Floating Hover Bar
    const floatingClasses = 'left-1/2 -translate-x-1/2 px-2 rounded-box bg-base-100/30 hover:bg-base-100/70 transition-[opacity,transform] duration-300 ease-in-out';
    
    if (toolbarPosition.value === 'bottom') {
       if (isHoverBottom.value || hasOpenMenu.value) {
          if (props.file.file_type === 2) {
            return `${commonClasses} ${floatingClasses} bottom-8 opacity-100`;
          } else {
            return `${commonClasses} ${floatingClasses} bottom-4 opacity-100`;
          }
       } else {
          if (props.file.file_type === 2) {
            return `${commonClasses} ${floatingClasses} bottom-8 opacity-0`;
          } else {
            return `${commonClasses} ${floatingClasses} bottom-4 opacity-0`;
          }
       }
    } else {
       if (isHoverTop.value || hasOpenMenu.value) {
          return `${commonClasses} ${floatingClasses} top-4 opacity-100`;
       } else {
          return `${commonClasses} ${floatingClasses} top-4 opacity-0`;
       }
    }
  }
});

const handleMenuOpenChange = (isOpen: boolean) => {
  hasOpenMenu.value = isOpen;
};

// Expose methods for parent component (ImageViewer)
const zoomIn = () => mediaRef.value?.zoomIn();
const zoomOut = () => mediaRef.value?.zoomOut();
const zoomActual = () => mediaRef.value?.zoomActual();
const rotateRight = () => mediaRef.value?.rotateRight();
const getViewportState = () => mediaRef.value?.getViewportState?.();
const applyViewportState = (viewport: any, silent = false) => mediaRef.value?.applyViewportState?.(viewport, silent);
const showMessage = (message: string, isWarning: boolean = false) => toolTipRef.value?.showTip(message, isWarning);
const showTip = (message: string, isWarning: boolean = false) => toolTipRef.value?.showTip(message, isWarning);

const triggerPrev = () => {
  if (props.hasPrevious) {
    emit('prev');
  } else {
    showTip((localeMsg.value as any).tooltip.image_viewer.first_image);
  }
}

const triggerNext = () => {
  if (props.hasNext) {
    emit('next');
  } else {
    showTip((localeMsg.value as any).tooltip.image_viewer.last_image);
  }
}

const handleToggleSlideShow = () => {
  if (!props.isSlideShow) {
    emit('update:isZoomFit', true);
  }
  emit('toggle-slide-show');
}

const handleMessageFromImageViewer = (payload: { message: string }) => {
  if (payload.message === 'prev') {
    triggerPrev();
  } else if (payload.message === 'next') {
    triggerNext();
  }
};

defineExpose({
  zoomIn,
  zoomOut,
  zoomActual,
  rotateRight,
  getViewportState,
  applyViewportState,
  showMessage,
  triggerPrev,
  triggerNext
});

const showFolderFiles = computed(() => {
  return !!(config.main.sidebarIndex === 0 && libConfig.album.id && libConfig.album.id !== 0);
});

const selectedFile = computed(() => props.file);

const singleFileMenuItems = computed(() => {
  if (props.mode === 2) return [];

  return useFileMenuItems(
    selectedFile,
    localeMsg,
    isMac,
    showFolderFiles,
    (action) => emit('item-action', { action, index: props.fileIndex })
  ).value;
});
</script>

<style scoped>
/* Disable text selection while dragging */
* {
  user-select: none;
}
 
@media (max-width: 600px) {
  #responsiveDiv {
    visibility: hidden;
  }
}
@media (min-width: 600px) {
  #responsiveDiv {
    visibility: visible;
  }
}
</style>
