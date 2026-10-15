import type { ChangedFile } from "../../lib/api";
import { useMessages } from "../../messages";
import rowStyles from "../../ui/ListRow.module.css";
import styles from "./ChangedFiles.module.css";

export function ChangedFiles({ files }: { files: ChangedFile[] }) {
  const m = useMessages();
  return (
    <ul className={rowStyles.list}>
      {files.map((file) => (
        <li key={`${file.origPath ?? ""}→${file.path}`} className={rowStyles.row}>
          <span className={`${styles.kind} ${styles[file.change]}`}>{m.changeKind[file.change]}</span>
          <span className={`${rowStyles.mono} ${rowStyles.grow}`} title={file.path}>
            {file.path}
          </span>
          {file.origPath && <span className={rowStyles.muted}>{m.renamedFrom(file.origPath)}</span>}
        </li>
      ))}
    </ul>
  );
}
