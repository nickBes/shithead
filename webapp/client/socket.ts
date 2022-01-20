import types from '@/bindings/bindings'

type OnUpdateCallback = (socket:Socket) => void
export type OnMessageCallback = (message:types.ServerMessage) => void

export interface UpdateObject {
    callback: OnUpdateCallback
    timeInterval: number
}

export default class Socket {
    connection: WebSocket;
    openMethod: (socket : Socket) => void
    updateInterval : number
    reconnectionTimout = 5000
    updateObject : UpdateObject

    // has to be run only in useEffect because we need the window to exist
    // to create a websocket and run updates
    constructor(serverUrl : string, updateObject : UpdateObject, onMessage:OnMessageCallback) {
        this.connection = new WebSocket(serverUrl);

        this.connection.onopen = () => this.update(updateObject)
        this.updateObject = updateObject

        this.connection.onmessage = (event : MessageEvent) => {
            // bad implementation by ts, will read that: 
            // https://dev.to/codeprototype/safely-parsing-json-to-a-typescript-interface-3lkj 
            // some day
            onMessage(JSON.parse(event.data) as types.ServerMessage)
        }

        this.connection.onclose
    }

    send(message : types.ClientMessage) {
        // might accidentally send a message while connecting
        if (this.connection.readyState == WebSocket.OPEN) {
            this.connection.send(JSON.stringify(message))
        }
    }

    update(updateObject : UpdateObject) {
        const socket = this
        this.updateInterval = window.setInterval(() => {
            updateObject.callback(socket)
        }, 
        updateObject.timeInterval)
    }

    stopUpdating() {
        clearInterval(this.updateInterval)
    }

    reconnect() {
        // stop doing stuff on the socket
        this.stopUpdating()
        // wait a lil to not kill cpu
        setTimeout(() => {
            this.connection = new WebSocket(this.connection.url)
            this.update(this.updateObject)
        }, this.reconnectionTimout)
    }
}