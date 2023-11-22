'use client';

import { useForm } from 'react-hook-form';
import AceEditor from 'react-ace';
import 'ace-builds/src-noconflict/mode-yaml';
// import 'ace-builds/src-noconflict/snippets/yaml';
import 'ace-builds/src-noconflict/theme-xcode';
import { memo } from 'react';

function EditorContainer(params: any) {
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

export default memo(EditorContainer);
