#!/usr/bin/env node
// Stand-in for `gentle-shell --mode rpc`: pi's JSONL rpc without any model.
//
// Reads one JSON command per line on stdin, writes one JSON event per line on
// stdout, and exits 0 when stdin ends. What a turn does depends on the prompt
// text, so a client can be driven deterministically:
//
//   prompt "error"  -> a provider failure plus a warning notice
//   prompt "dialog" -> confirm, select and input dialogs, one at a time
//   prompt "slow"   -> text deltas every 100 ms until `abort`
//   anything else   -> "echo: <prompt>"
//
// Event shapes follow the recorded streams in
// crates/gentle-core/tests/fixtures/. It runs no model, reads no files and
// writes nothing but stdout.

import * as readline from 'node:readline';
import { helpersFrame } from './helpers-frame.mjs';

/** Lines like the fixtures: one JSON object per line, LF-terminated. */
function send(event) {
  process.stdout.write(JSON.stringify(event) + '\n');
}

/** A counter so `usage` grows the way the recorded deltas do. */
let output = 0;

function usage() {
  return {
    input: 10,
    output,
    cacheRead: 0,
    cacheWrite: 0,
    totalTokens: 10 + output,
    cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0, total: 0 },
  };
}

function update(assistantMessageEvent) {
  send({ type: 'message_update', usage: usage(), assistantMessageEvent });
}

function agentStart() {
  send({ type: 'agent_start' });
}

function messageStart() {
  send({ type: 'message_start', message: { role: 'assistant', content: [] } });
}

function textStart(contentIndex) {
  update({ type: 'text_start', contentIndex });
}

function textDelta(contentIndex, delta) {
  output += 1;
  update({ type: 'text_delta', contentIndex, delta });
}

function textEnd(contentIndex, content) {
  update({ type: 'text_end', contentIndex, content });
}

/** Closes the assistant message with `text` as its single text block. */
function messageEnd(text, extra = {}) {
  send({
    type: 'message_end',
    message: {
      role: 'assistant',
      content: [{ type: 'text', text }],
      stopReason: 'stop',
      ...extra,
    },
  });
}

function agentEnd() {
  send({ type: 'agent_end', messages: [], willRetry: false });
  send({ type: 'agent_settled' });
}

/** The default turn: the reply is the prompt, prefixed. */
function echoTurn(message) {
  const text = 'echo: ' + message;
  agentStart();
  messageStart();
  textStart(0);
  textDelta(0, text);
  textEnd(0, text);
  messageEnd(text);
  agentEnd();
}

/** Publishes the helpers widget, then answers like an echo. */
function helpersTurn() {
  send({
    type: 'extension_ui_request',
    id: 'helpers-widget',
    method: 'setWidget',
    widgetKey: 'gentle-agents',
    widgetLines: [JSON.stringify(helpersFrame())],
  });
  return echoTurn('helpers');
}

/** A provider failure: the message carries an error, then a notice arrives. */
function errorTurn() {
  agentStart();
  messageStart();
  messageEnd('Provider says no.', {
    stopReason: 'error',
    errorMessage: 'quota exceeded (fake)',
  });
  send({
    type: 'extension_ui_request',
    id: 'notice-1',
    method: 'notify',
    notifyType: 'warning',
    message: 'rate limit reached (fake)',
  });
  agentEnd();
}

/** Dialog ids whose answer this process is still waiting for. */
const waiting = new Map();

/** Asks one blocking dialog and resolves with the raw response the client sends. */
function ask(request) {
  return new Promise((resolve) => {
    waiting.set(request.id, resolve);
    send({ type: 'extension_ui_request', ...request });
  });
}

/** Three blocking dialogs, in order, then their answers as the reply text. */
async function dialogTurn() {
  agentStart();
  const answers = [];
  answers.push(
    await ask({
      id: 'd1',
      method: 'confirm',
      title: 'Allow this command?',
      message: 'Run `ls`?',
      timeout: 10000,
    }),
  );
  answers.push(
    await ask({
      id: 'd2',
      method: 'select',
      title: 'Pick a letter',
      options: ['a', 'b'],
      timeout: 10000,
    }),
  );
  answers.push(
    await ask({
      id: 'd3',
      method: 'input',
      title: 'Your name',
      placeholder: 'name',
      timeout: 10000,
    }),
  );
  const text = 'RESPONSES: ' + JSON.stringify(answers);
  messageStart();
  textStart(0);
  textDelta(0, text);
  textEnd(0, text);
  messageEnd(text);
  agentEnd();
}

/** Set while a `slow` turn is streaming; calling it ends that stream. */
let endStream = null;

/** Streams "tick " every 100 ms until `abort` (or 30 s, so nothing hangs). */
function slowTurn() {
  return new Promise((resolve) => {
    let streamed = '';
    let done = false;
    const finish = () => {
      if (done) return;
      done = true;
      clearInterval(pump);
      clearTimeout(safety);
      endStream = null;
      textEnd(0, streamed);
      messageEnd('ABORTED');
      agentEnd();
      resolve();
    };
    agentStart();
    messageStart();
    textStart(0);
    const pump = setInterval(() => {
      streamed += 'tick ';
      textDelta(0, 'tick ');
    }, 100);
    const safety = setTimeout(finish, 30000);
    endStream = finish;
  });
}

function turn(message) {
  switch (message) {
    case 'error':
      return errorTurn();
    case 'dialog':
      return dialogTurn();
    case 'slow':
      return slowTurn();
    case 'helpers':
      return helpersTurn();
    default:
      return echoTurn(message);
  }
}

/** Turns run one at a time, in arrival order. */
let queue = Promise.resolve();

function onCommand(line) {
  if (line.trim() === '') return;
  let command;
  try {
    command = JSON.parse(line);
  } catch {
    return; // malformed input is ignored, never a crash
  }
  switch (command.type) {
    case 'prompt': {
      const message = typeof command.message === 'string' ? command.message : '';
      // Always acknowledge first: the turn's events come after it.
      send({
        type: 'response',
        command: 'prompt',
        success: true,
        data: { disposition: 'started' },
      });
      queue = queue.then(() => turn(message));
      return;
    }
    case 'abort': {
      if (endStream) endStream();
      return;
    }
    case 'extension_ui_response': {
      const resolve = waiting.get(command.id);
      if (resolve) {
        waiting.delete(command.id);
        resolve(command);
      }
      return;
    }
    default: {
      // new_session, get_state, ...: accepted, with nothing this stand-in models.
      send({ type: 'response', command: command.type, success: true, data: {} });
    }
  }
}

const input = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
input.on('line', onCommand);
// stdin ended: per the rpc framing rules (and gentle-term's shutdown), exit 0.
input.on('close', () => process.exit(0));
