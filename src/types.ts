export type Condition =
  | { kind: "hostEquals"; value: string }
  | { kind: "domainSuffix"; value: string }
  | { kind: "hostWildcard"; value: string }
  | { kind: "pathPrefix"; value: string }
  | { kind: "pathRegex"; value: string }
  | { kind: "queryParameter"; name: string; value?: string }
  | { kind: "scheme"; value: string }
  | { kind: "port"; value: number }
  | { kind: "localhost" }
  | { kind: "privateNetwork" };
export interface Action { browserId: string; profile?: string; private: boolean }
export interface Rule { id: string; name: string; enabled: boolean; priority: number; conditionMode: "all" | "any"; conditions: Condition[]; action: Action; createdAt: string; updatedAt: string }
export interface Profile { id: string; name: string; path?: string }
export interface Browser { id: string; name: string; executable: string; kind: string; profiles: Profile[] }
export interface Config { version: number; rules: Rule[]; browsers: Browser[]; settings: { historyEnabled: boolean; storeFullUrls: boolean; launchAtLogin: boolean; pausedUntil?: string }; history: { timestamp: string; host: string; browserId: string }[] }
export interface Simulation { normalizedUrl: string; matches: Rule[]; result?: Action }
