import { computed, ref, type Ref } from 'vue';
import { check, type Update, type DownloadEvent } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

const UPDATE_CHECK_INTERVAL = 24 * 60 * 60 * 1000;
const UPDATE_CHECK_KEY = 'lap_last_update_check';

function getErrorMessage(error: unknown, fallback: string) {
  if (typeof error === 'string' && error.trim()) return error;
  if (error && typeof error === 'object' && 'message' in error && typeof (error as any).message === 'string') {
    return (error as any).message;
  }
  return fallback;
}

export function useAppUpdater(
  localeMsg: Ref<any>,
  toolTipRef: Ref<{ showTip: (message: string, isWarning?: boolean) => void } | null>
) {
  const updateAvailable = ref(false);
  const isCheckingUpdate = ref(false);
  const isInstallingUpdate = ref(false);
  const isDownloadingUpdate = ref(false);
  const isUpdateReadyToRestart = ref(false);
  const updateVersion = ref('');
  const downloadPercent = ref<number | null>(null);
  let downloadTotalBytes = 0;
  let downloadedBytes = 0;
  let currentUpdate: Update | null = null;

  const restartLabel = computed(() => localeMsg.value.settings.about.auto_update.restart);
  const downloadProgressLabel = computed(() => {
    if (downloadPercent.value === null) {
      return localeMsg.value.settings.about.auto_update.downloading_update;
    }
    return localeMsg.value.settings.about.auto_update.downloading.replace(
      '{percent}',
      String(downloadPercent.value)
    );
  });

  const updateButtonTooltip = computed(() => {
    if (isDownloadingUpdate.value) {
      return downloadProgressLabel.value;
    }
    if (isInstallingUpdate.value) {
      return localeMsg.value.settings.about.auto_update.installing;
    }
    if (isCheckingUpdate.value) {
      return localeMsg.value.settings.about.auto_update.checking;
    }
    if (isUpdateReadyToRestart.value) {
      return restartLabel.value;
    }
    if (updateAvailable.value && updateVersion.value) {
      return localeMsg.value.settings.about.auto_update.new_version_available.replace('{version}', updateVersion.value);
    }
    return localeMsg.value.settings.about.auto_update.check;
  });

  const updateButtonText = computed(() => {
    if (isDownloadingUpdate.value) return downloadProgressLabel.value;
    if (isInstallingUpdate.value) return localeMsg.value.settings.about.auto_update.installing;
    if (isCheckingUpdate.value) return localeMsg.value.settings.about.auto_update.checking;
    if (isUpdateReadyToRestart.value) return restartLabel.value;
    if (updateAvailable.value) return localeMsg.value.settings.about.auto_update.update;
    return localeMsg.value.settings.about.auto_update.check;
  });

  const isUpdateActionEnabled = computed(() =>
    updateAvailable.value || isUpdateReadyToRestart.value
  );

  function resetDownloadProgress() {
    isDownloadingUpdate.value = false;
    downloadPercent.value = null;
    downloadTotalBytes = 0;
    downloadedBytes = 0;
  }

  function handleDownloadEvent(event: DownloadEvent) {
    if (event.event === 'Started') {
      isDownloadingUpdate.value = true;
      downloadTotalBytes = event.data.contentLength || 0;
      downloadedBytes = 0;
      downloadPercent.value = downloadTotalBytes > 0 ? 0 : null;
      toolTipRef.value?.showTip(localeMsg.value.settings.about.auto_update.downloading_started);
      return;
    }

    if (event.event === 'Progress') {
      downloadedBytes += event.data.chunkLength;
      if (downloadTotalBytes > 0) {
        downloadPercent.value = Math.min(100, Math.round((downloadedBytes / downloadTotalBytes) * 100));
      }
      return;
    }

    if (event.event === 'Finished') {
      isDownloadingUpdate.value = false;
      downloadPercent.value = 100;
      toolTipRef.value?.showTip(localeMsg.value.settings.about.auto_update.download_finished);
    }
  }

  async function checkForUpdates(manual = false) {
    if (isCheckingUpdate.value || isInstallingUpdate.value) return;

    if (!manual) {
      const lastCheck = localStorage.getItem(UPDATE_CHECK_KEY);
      if (lastCheck && Date.now() - Number(lastCheck) < UPDATE_CHECK_INTERVAL) {
        return;
      }
    }

    isCheckingUpdate.value = true;
    updateAvailable.value = false;
    updateVersion.value = '';
    currentUpdate = null;
    resetDownloadProgress();

    try {
      const update = await check();
      localStorage.setItem(UPDATE_CHECK_KEY, String(Date.now()));
      if (!update) {
        if (manual) {
          toolTipRef.value?.showTip(localeMsg.value.settings.about.auto_update.latest_version);
        }
        return;
      }

      updateAvailable.value = true;
      updateVersion.value = update.version;
      currentUpdate = update;
      toolTipRef.value?.showTip(
        localeMsg.value.settings.about.auto_update.new_version_available.replace('{version}', update.version)
      );
    } catch (error: unknown) {
      const message = getErrorMessage(error, localeMsg.value.settings.about.auto_update.failed_check);
      console.error('Failed to check for updates:', error);
      if (manual) {
        toolTipRef.value?.showTip(message, true);
      }
    } finally {
      isCheckingUpdate.value = false;
    }
  }

  async function installAvailableUpdate() {
    if (isInstallingUpdate.value) return;

    if (isUpdateReadyToRestart.value) {
      try {
        await relaunch();
      } catch (error: unknown) {
        const message = getErrorMessage(error, localeMsg.value.settings.about.auto_update.failed_install);
        console.error('Failed to relaunch after update:', error);
        toolTipRef.value?.showTip(message, true);
      }
      return;
    }

    if (!currentUpdate) return;

    try {
      isInstallingUpdate.value = true;
      resetDownloadProgress();
      toolTipRef.value?.showTip(localeMsg.value.settings.about.auto_update.downloading_update);
      await currentUpdate.downloadAndInstall(handleDownloadEvent);
      updateAvailable.value = false;
      updateVersion.value = '';
      currentUpdate = null;
      isUpdateReadyToRestart.value = true;
      toolTipRef.value?.showTip(localeMsg.value.settings.about.auto_update.update_installed_waiting_restart);
    } catch (error: unknown) {
      resetDownloadProgress();
      const message = getErrorMessage(error, localeMsg.value.settings.about.auto_update.failed_install);
      console.error('Failed to install update:', error);
      toolTipRef.value?.showTip(message, true);
    } finally {
      resetDownloadProgress();
      isInstallingUpdate.value = false;
    }
  }

  async function handleUpdateAction() {
    if (isInstallingUpdate.value || isCheckingUpdate.value) return;
    if (updateAvailable.value || isUpdateReadyToRestart.value) {
      await installAvailableUpdate();
      return;
    }
    await checkForUpdates(true);
  }

  return {
    updateAvailable,
    isCheckingUpdate,
    isInstallingUpdate,
    isUpdateReadyToRestart,
    updateVersion,
    isDownloadingUpdate,
    downloadPercent,
    updateButtonTooltip,
    updateButtonText,
    downloadProgressLabel,
    isUpdateActionEnabled,
    checkForUpdates,
    installAvailableUpdate,
    handleUpdateAction,
  };
}
