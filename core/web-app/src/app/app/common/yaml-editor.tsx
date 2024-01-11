import AceEditor from 'react-ace';
import 'ace-builds/src-noconflict/ext-language_tools';
import 'ace-builds/src-noconflict/theme-xcode';
import 'ace-builds/src-noconflict/mode-yaml';
import { Ace } from 'ace-builds';

export default function YAMLEditor({
  value,
  readonly,
  showLineNumbers = true,
  annotations,
  onChange,
}: {
  value: string;
  readonly?: boolean;
  showLineNumbers?: boolean;
  annotations?: Ace.Annotation[];
  onChange?: (...event: any[]) => void;
}) {
  return (
    <AceEditor
      mode="yaml"
      theme="xcode"
      onChange={onChange}
      value={value}
      readOnly={readonly}
      tabSize={2}
      annotations={annotations}
      wrapEnabled={true}
      editorProps={{
        $blockScrolling: true,
      }}
      setOptions={{
        tabSize: 2,
        showLineNumbers,
        showGutter: showLineNumbers,
        // TODO: Autocomplete
        // https://github.com/ajaxorg/ace/wiki/How-to-enable-Autocomplete-in-the-Ace-editor
        enableBasicAutocompletion: true,
        enableLiveAutocompletion: true,
        enableSnippets: true,
      }}
      style={{
        width: '100%',
        height: '100%',
        minHeight: '40rem',
      }}
    />
  );
}
