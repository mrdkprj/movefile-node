const { contextBridge, ipcRenderer } = require('electron')

contextBridge.exposeInMainWorld('electronAPI', {
  setTitle: (s,d) => ipcRenderer.send('set-title', s, d),
  toggle: () => ipcRenderer.send('toggle'),
  append: (s) => ipcRenderer.send('append', s),
  reload: () => ipcRenderer.send('reload'),
  onMyEventName: (callback) => ipcRenderer.on('progress', (_e, ...args) => callback(args)),
})