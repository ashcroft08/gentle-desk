/* global console, process, fetch, WebSocket */
// Real-window probe driver for the a11y lab (T2c). Drives the app's WebView2
// over the Chrome DevTools Protocol exposed by
// WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port; it changes no
// source: the start URL (with the lab hash) is set through CDP navigation.
//
// Usage:
//   node probe.mjs goto <url>            - navigate the webview to a URL+hash
//   node probe.mjs open <bitsui|meltui>  - open the three lab widgets in order
//   node probe.mjs location              - print the current URL (sanity)
//
// Node >= 22 (global WebSocket).
const port = Number(process.env.CDP_PORT ?? '9222')

async function pageTarget() {
  const list = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json()
  const page = list.find((t) => t.type === 'page' && !t.url.startsWith('devtools:'))
  if (!page) return null
  return { ws: page.webSocketDebuggerUrl, url: page.url }
}

function cdp(wsUrl) {
  let id = 0
  const ws = new WebSocket(wsUrl)
  const pending = new Map()
  ws.onmessage = (msg) => {
    const data = JSON.parse(String(msg.data))
    if (data.id && pending.has(data.id)) {
      const { resolve, reject } = pending.get(data.id)
      pending.delete(data.id)
      if (data.error) reject(new Error(JSON.stringify(data.error)))
      else resolve(data.result)
    }
  }
  const opened = new Promise((resolve, reject) => {
    ws.onopen = () => resolve(true)
    ws.onerror = () => reject(new Error('websocket connect failed'))
  })
  return {
    async send(method, params = {}) {
      await opened
      id += 1
      const mid = id
      return new Promise((resolve, reject) => {
        pending.set(mid, { resolve, reject })
        ws.send(JSON.stringify({ id: mid, method, params }))
      })
    },
    close() {
      ws.close()
    }
  }
}

async function main() {
  const [, , cmd, arg] = process.argv
  const target = await pageTarget()
  if (!target) {
    console.error('probe.mjs: no page target found; is the app running with --remote-debugging-port?')
    process.exit(2)
  }
  const conn = cdp(target.ws)
  await conn.send('Runtime.enable')
  await conn.send('Page.enable')

  if (cmd === 'goto' || cmd === 'location') {
    if (cmd === 'goto') {
      await conn.send('Page.navigate', { url: arg })
      console.log(`navigated to ${arg}`)
    } else {
      const r = await conn.send('Runtime.evaluate', {
        expression: 'location.href',
        returnByValue: true
      })
      console.log(r.result.value)
    }
  } else if (cmd === 'openlist') {
    // open only the list popup (single-widget probe state), then wait
    const script = `(async () => {
      const wait = (ms) => new Promise((r) => setTimeout(r, ms))
      const byText = (t) =>
        Array.from(document.querySelectorAll('button')).find(
          (b) => (b.textContent || '').trim() === t
        )
      const el = byText('Alpha')
      if (!el) return 'missing trigger: Alpha'
      el.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }))
      el.click()
      await wait(1500)
      return 'list open? ' + (el.getAttribute('aria-expanded') ?? 'no-attr')
    })()`
    const r = await conn.send('Runtime.evaluate', {
      expression: script,
      awaitPromise: true,
      returnByValue: true
    })
    console.log(r.result.value)
  } else if (cmd === 'openkeys3') {
    // keyboard-open the list, then pointer-open the menu (only the pointer
    // down on top of the closed menu trigger), then Escape the dialog away
    const script = `(async () => {
      const wait = (ms) => new Promise((r) => setTimeout(r, ms))
      const byText = (t) =>
        Array.from(document.querySelectorAll('button')).find(
          (b) => (b.textContent || '').trim() === t
        )
      const openList = byText('Alpha')
      openList.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
      await wait(1200)
      const openMenu = byText('Actions')
      openMenu.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }))
      openMenu.click()
      await wait(1200)
      return 'done'
    })()`
    const r = await conn.send('Runtime.evaluate', {
      expression: script,
      awaitPromise: true,
      returnByValue: true
    })
    console.log(r.result.value)
  } else if (cmd === 'openkeys2') {
    // Close whatever popovers are open, then open list + menu with keyboard
    // events (no pointer events, so no outside-click closes) and the dialog's
    // own Escape handoff first.
    const script = `(async () => {
      const wait = (ms) => new Promise((r) => setTimeout(r, ms))
      const byText = (t) =>
        Array.from(document.querySelectorAll('button')).find(
          (b) => (b.textContent || '').trim() === t
        )
      document.activeElement?.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
      await wait(1200)
      const opened = []
      for (const label of ['Alpha', 'Actions']) {
        const el = byText(label)
        if (!el) return 'missing trigger: ' + label
        el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
        await wait(1200)
        opened.push(el.textContent.trim())
      }
      return 'keyboard-opened: ' + opened.join(', ')
    })()`
    const r = await conn.send('Runtime.evaluate', {
      expression: script,
      awaitPromise: true,
      returnByValue: true
    })
    console.log(r.result.value)
  } else if (cmd === 'openkeys') {
    // Keyboard-only opening for the single permitted probe of a candidate:
    // opening a popover with a pointer event dispatches the outside-click
    // close of the popovers opened before it, so a probe taken after a
    // pointer sequence only sees the last widget. Keyboard-opened popovers
    // do not emit outside-pointer events, so all three stay open.
    const candidate = arg
    const labels =
      candidate === 'bitsui'
        ? ['Alpha', 'Actions', 'Open dialog']
        : ['Actions', 'Alpha', 'Open dialog']
    const script = `(async () => {
      const labels = ${JSON.stringify(labels)}
      const byText = (t) =>
        Array.from(document.querySelectorAll('button')).find(
          (b) => (b.textContent || '').trim() === t
        )
      const wait = (ms) => new Promise((r) => setTimeout(r, ms))
      const opened = []
      for (const label of labels) {
        const el = byText(label)
        if (!el) return 'missing trigger: ' + label
        el.focus()
        await wait(400)
        el.dispatchEvent(
          new KeyboardEvent('keydown', { key: 'Enter', bubbles: true })
        )
        await wait(1200)
        opened.push(el.textContent.trim())
      }
      return 'keyboard-opened: ' + opened.join(', ')
    })()`
    const r = await conn.send('Runtime.evaluate', {
      expression: script,
      awaitPromise: true,
      returnByValue: true
    })
    console.log(r.result.value)
  } else if (cmd === 'open') {
    const candidate = arg
    // Open the menu and the list first, then the (modal) dialog last: the
    // dialog's focus scope is the last thing to engage before the single UIA
    // probe of this candidate.
    const labels =
      candidate === 'bitsui'
        ? ['Alpha', 'Actions', 'Open dialog']
        : ['Actions', 'Alpha', 'Open dialog']
    const script = `(async () => {
      const labels = ${JSON.stringify(labels)}
      const byText = (t) =>
        Array.from(document.querySelectorAll('button')).find(
          (b) => (b.textContent || '').trim() === t
        )
      const wait = (ms) => new Promise((r) => setTimeout(r, ms))
      const opened = []
      for (const label of labels) {
        const el = byText(label)
        if (!el) return 'missing trigger: ' + label
        el.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }))
        el.click()
        opened.push(el.textContent.trim())
        await wait(1200)
      }
      return 'opened: ' + opened.join(', ')
    })()`
    const r = await conn.send('Runtime.evaluate', {
      expression: script,
      awaitPromise: true,
      returnByValue: true
    })
    console.log(r.result.value)
  } else if (cmd === 'dump') {
    const r = await conn.send('Runtime.evaluate', {
      expression: `(() => {
        const open = (sel, attr) => {
          const el = document.querySelector(sel)
          if (!el) return 'absent'
          return el.getAttribute(attr) ?? 'no-attr'
        }
        const w = document.querySelector('[data-bits-floating-content-wrapper]')
        const bitsVis = w ? getComputedStyle(w).visibility : '-'
        return JSON.stringify({
          bitsDialog: open('[data-dialog-content]', 'data-state'),
          bitsMenu: open('[data-dropdown-menu-content]', 'data-state'),
          bitsSelect: open('[data-select-content]', 'data-state'),
          meltDialog: open('[data-melt-dialog-content]', 'data-state'),
          meltMenu: open('[data-melt-dropdown-menu-content]', 'data-state'),
          meltSelect: open('[data-melt-select-menu]', 'hidden'),
          meltList: open('[role=listbox]', 'hidden'),
          meltOptions: document.querySelectorAll('[role=option]').length,
          meltMenuitems: document.querySelectorAll('[role=menuitem]').length,
          meltDialogOpen: open('[role=dialog][data-melt-dialog-content]', 'data-state')
        })
      })()`,
      returnByValue: true
    })
    console.log(r.result.value)
  } else {
    console.error('usage: node probe.mjs goto <url> | open <bitsui|meltui> | location')
    process.exit(2)
  }
  conn.close()
}

main().catch((e) => {
  console.error('probe.mjs failed:', e instanceof Error ? e.message : e)
  process.exit(1)
})
