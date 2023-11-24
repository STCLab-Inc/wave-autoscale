'use client';

import React, { memo } from 'react';

import AceEditor from 'react-ace';
import 'ace-builds/src-noconflict/ext-language_tools';
import 'ace-builds/src-noconflict/mode-javascript';
import 'ace-builds/src-noconflict/snippets/javascript';
import 'ace-builds/src-noconflict/theme-xcode';
import 'ace-builds/src-noconflict/mode-yaml';

function codeEditor(params: any) {
  const { onChange, value, annotations } = params;

  return (
    <AceEditor
      mode="yaml"
      theme="xcode"
      onChange={onChange}
      value={value}
      editorProps={{
        $blockScrolling: true,
      }}
      setOptions={{
        showLineNumbers: true,
        // TODO: Autocomplete
        // https://github.com/ajaxorg/ace/wiki/How-to-enable-Autocomplete-in-the-Ace-editor
        enableBasicAutocompletion: true,
        enableLiveAutocompletion: true,
        enableSnippets: true,
        showGutter: true,
      }}
      style={{
        width: '100%',
        height: '100%',
        minHeight: '200px',
      }}
      annotations={annotations}
    />
  );
}

export default memo(codeEditor);
