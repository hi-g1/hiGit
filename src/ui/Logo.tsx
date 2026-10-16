import logoDark from "../../assets/brand/logo-dark.svg";
import logoLight from "../../assets/brand/logo-light.svg";
import styles from "./Logo.module.css";

/**
 * hiGit 로고(D-163). 브랜드 SVG 파일을 그대로 쓰고 색을 다시 칠하지 않는다. 라이트 모드는 logo-light, 다크 모드는
 * logo-dark이고 어느 쪽이 보일지는 화면 모드와 같은 규칙(CSS)으로 정한다.
 */
export function Logo({ size }: { size: "start" | "settings" }) {
  const height = size === "start" ? styles.start : styles.settings;
  return (
    <span className={`${styles.logo} ${height}`}>
      <img className={styles.light} src={logoLight} alt="hiGit" />
      <img className={styles.dark} src={logoDark} alt="hiGit" />
    </span>
  );
}
