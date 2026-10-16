# S04: Windows 관리 runtime 정상 활성 GUI 인수

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: root 새 scripts/windows_runtime_positive_tools_probe.py·scripts/windows_runtime_positive_tools_observer.cs·전용 증거/문서; soak_recovery_audit /tmp draft만
- 선행 작업: S04-managed-runtime-ui-integration·기존 Windows runtime wrong-version/Use 인수와 최신CPU 전체537소스 검증

## 사용자 결과

격리 Windows GUI에서 비활성 관리 Deno/FFmpeg의 정상 버전을 검증·활성화하고 각 다운로드 필드에 정확한 실행 파일 경로를 전달하는 실제 동작을 확인한다.

## 계약과 범위

새 owned TEMP/data의 합성 C# runtime1.0.0/2.0.0과 bounded synthetic package만 사용한다. 실제 공식 공급/자동설치채택/실제사이트는 이 시험이 아니다. UIA/원PID FILETIME/HWND/private desktop/ownedJob/120초/원화면복귀/no globalinput·clipboard는 기존 observer의 계약을 유지한다. 생산 Rust/Cargo/schema 변경0. root가 원source/binary SHA·prioractive와 선택후2.0.0 active DB·fullfile SHA·외부 sentinel·정확Use field readback·Job自然0를 통합 검토한다.

## 인수 조건

Deno/FFmpeg 탭에서 runtime설치disabled·공급pending 유지, 정상2.0.0 활성 후 다운로드에사용 exact typedpath와 active DB2.0.0 검증. fixture executable/notice/manifest 원bytes 보존. 실패원raw는 남기고 수정후 새evidence를 사용한다. physicalIME/reader/공식binary trust는 이 인수를 대신하지 않는다.

## 검증과 handoff

기존 runner의 seeded도 wrong-version reporter1.0.0만 사용했으므로 이를 정상inactive활성 PASS로 확대하지 않는다. 신규positivecollector를 별도로 준비한다.

실제 새collector compile과WindowsGUI PASS(windows-runtime-positive-first-20261005-continue). 원합성1.0active→정상2.0actualversion확인→Deno/FFmpeg 각activeDB2.0→Use exact 다운로드field 두개를검증했다. root는 원20여fileSHA/SQLiteintegrity/typedpath raw/sentinel/naturalJob0/privateclose/input불변과field2PNG를독립확인했다. 기존wrongversion은별도원시험이며 자동공급·실제공식runtime·사이트·물리reader를검증한것이아니다. root 최초logUTF8읽기는nativeConsole인코딩으로읽기오류였고 원ASCIIterminal을bytes로재확인했다;원GUI실패나제품오류로오인하지않는다. 독립하위raw검토는후속첨부한다. 생산Rust/schema/Cargo변경0.

독립하위검토도PASS:原file20개·당시538sourceSHA·two active2.0·typedpath2개·naturalJob0/private/input을대조했다. 이후실험example1개변경은당시GUI원snapshot과구분해기록했다.
