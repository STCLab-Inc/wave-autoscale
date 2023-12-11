import AceEditor from 'react-ace';
import 'ace-builds/src-noconflict/ext-language_tools';
import 'ace-builds/src-noconflict/theme-xcode';
import 'ace-builds/src-noconflict/mode-yaml';

export default function YAMLEditor({
  value,
  onChange,
}: {
  value: string;
  onChange: (...event: any[]) => void;
}) {
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
        tabSize: 2,
        showLineNumbers: false,
        // TODO: Autocomplete
        // https://github.com/ajaxorg/ace/wiki/How-to-enable-Autocomplete-in-the-Ace-editor
        enableBasicAutocompletion: true,
        enableLiveAutocompletion: true,
        enableSnippets: true,
        showGutter: false,
      }}
      style={{
        width: '100%',
        height: '100%',
        minHeight: '40rem',
      }}
    />
  );
}
