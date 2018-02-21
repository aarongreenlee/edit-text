import 'bootstrap/dist/css/bootstrap.min.css';
import './mote.scss';
import * as commands from './commands';
import Editor from './editor';
import Parent from './parent';
import * as interop from './interop';

import $ from 'jquery';
import bootstrap from 'bootstrap';
import bootbox from 'bootbox';

// Consume bootstrap so bootbox works.
bootstrap;


declare var WebAssembly: any;
declare var TextEncoder: any;
declare var TextDecoder: any;

// Entry.
if (document.body.id == 'multi') {
  document.body.innerHTML = `

<h1>Mercutio
  <button id="action-monkey">🙈🙉🙊</button>
  <span style="font-family: monospace; padding: 3px 5px;" id="timer"></span>
</h1>

<table id="clients">
  <tr>
    <td>
      <iframe src="/client/?a"></iframe>
    </td>
    <td>
      <iframe src="/client/?b"></iframe>
    </td>
    <td>
      <iframe src="/client/?c"></iframe>
    </td>
  </tr>
</table>

`;

  new Parent();
}
else if (document.body.id == 'client') {
  document.body.innerHTML = `
<div id="toolbar">
  <div id="native-buttons"></div>
  <div id="local-buttons"></div>
</div>
<div class="mote" id="mote"></div>
`;


  if (window.parent != window) {
    // Blur/Focus classes.
    $(window).on('focus', () => $(document.body).removeClass('blurred'));
    $(window).on('blur', () => $(document.body).addClass('blurred'));
    $(document.body).addClass('blurred');
  }

  let editor = new Editor(document.getElementById('mote'), '$$$$$$');

  console.log('start');

  // Use cross-compiled WASM bundle.
  let WASM = true;
  if (!WASM) {
    editor.syncConnect();
    editor.nativeConnect();
  } else {
    interop.instantiate(function (data) {
      // console.log('----> js_command:', data);

      // Make this async so we don't have deeply nested call stacks from Rust<->JS interop.
      setImmediate(() => {
        editor.onNativeMessage({
          data: data,
        });
      });
    })
    .then(Module => {
      Module.wasm_setup();

      setImmediate(() => {
        // Websocket port
        let url = window.location.host.match(/localhost/) ?
          'ws://' + window.location.host.replace(/:\d+$|$/, ':8001') + '/' :
          'ws://' + window.location.host + '/ws/';

        let syncSocket = new WebSocket(url);
        editor.Module = Module; 
        editor.syncSocket = syncSocket;
        syncSocket.onopen = function (event) {
          console.log('Editor "%s" is connected.', editor.editorID);
        };

        // Keepalive
        setInterval(() => {
          syncSocket.send(JSON.stringify({
            Keepalive: null,
          }));
        }, 1000);

        syncSocket.onmessage = function (event) {
          // console.log('GOT SYNC SOCKET MESSAGE:', event.data);
          Module.wasm_command({
            SyncClientCommand: JSON.parse(event.data),
          });
        };
        syncSocket.onclose = function () {
          // TODO use a class
          $('html').css('background', '#f00');
          $('body').css('opacity', '0.7');
        }
      });
    })
  }
} else {
  document.body.innerHTML = '404';
}