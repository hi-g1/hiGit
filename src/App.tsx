import { Term } from "./ui/Term";

export function App() {
  return (
    <main>
      <h1>hiGit</h1>
      <p>
        <Term id="status" /> · <Term id="log" />
      </p>
      <code>a1b2c3d src/한글 파일.txt</code>
    </main>
  );
}
