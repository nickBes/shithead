import { types } from '@/bindings/bindings'

export const parseMessageEvent = (event : MessageEvent<any>) => {
    // bad implementation by ts, will read that: 
    // https://dev.to/codeprototype/safely-parsing-json-to-a-typescript-interface-3lkj 
    // some day
    return JSON.parse(event.data) as types.ServerMessage
}

export const clientMessageToJSON = (message : types.ClientMessage) => {
    return JSON.stringify(message)
}