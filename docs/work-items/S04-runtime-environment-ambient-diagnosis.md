# S04: Windows 실제 ambient 환경 encoder 실패 진단

- 상태: done (환경 분류 관측 범위)
- 분류: fix
- 우선순위: P0
- 요구사항: R04, R05, R08
- 담당: metrics_recovery
- 수정 파일: /tmp/windows_runtime_environment_ambient_probe.cs, 이 문서 및 새 전용 evidence
- 선행 작업: encoder pure7PASS/native PreparedChild 환경 거절 실제 실패

## 범위와 인수 조건

제품 Rust/Cargo/schema는 읽기·동결 유지한다. 실제 Windows GetEnvironmentStringsW 원 block을 project private native CLI host에서 관측하며 변수 이름·값을 출력하지 않는다. drive pseudoentry/empty/equal/space/nonASCII/control/key 길이, UTF-16 value 오류/길이, casefold 중복, 전체 UTF-16 길이와 개수만 bounded JSON으로 남긴다. .NET Environment dictionary로 원 pseudoentry를 누락시키지 않는다. source bytes/원 private desktop/input 불변을 확인하고 관측만으로 제품 수정 성공을 주장하지 않는다.

raw block65536units/변수4096 상한, 보조 helper compile/run 각20초이며 사용자 desktop/clipboard/설정/쿠키 데이터에 접근하지 않는다. 기존 configure를 같은 방식으로 사용한다. 환경 값을 logs/config로 복제하지 않고 기존 host가 소유한 프로세스만 종료·정리한다.

## 검증과 인계

`python3 /tmp/run_ambient_categories.py`를 기존 private `windows-cli-host.exe`와 native csc로 실행했다. 첫 실행은 호스트의 추가 argv 요구로 usage만 출력하여 실패했다. [첫 원시 실패](../validation/evidence/windows-runtime-environment-ambient-first/categories.log)를 보존했으며 제품 실패 재현으로 세지 않는다. 고정 `--categories` argv를 추가한 [두 번째 결과](../validation/evidence/windows-runtime-environment-ambient-second/result.json)와 분류를 세분화한 [세 번째 결과](../validation/evidence/windows-runtime-environment-ambient-third/result.json)는 실제 exit 0/PASS다.

원 block에는 변수 57개, 종료 NUL 포함 UTF-16 3906 units가 있었다. 선두 `=`를 포함하지만 drive pseudoentry 형식이 아닌 키가 정확히 1개였고 drive 형식은 0개였다. 키 이름과 값은 출력하거나 기록하지 않았다. casefold 중복, 공백/nonASCII/control 키, 잘못된 surrogate, NUL, 길이·개수 상한 초과는 모두 0개다. 따라서 현재 환경에서 drive 전용 가설보다 기타 선두 `=` pseudoentry 거절 가설을 우선한다. 이 관측은 원 native 환경 block의 분류이고 Rust `vars_os()`와 정확히 같은 항목을 전달한다는 증거는 아니다. 제품 encoder의 고정 오류 variant 계측은 주 담당이 별도로 수행한다.

세 번째 C# source SHA256는 `35471fabee9da2b4473370b67007342326b1fbc5ff6413b162e460d28dddc545`, native exe SHA256는 `b78dafe856b39bf6722eac95f4340b95d0ad1e319ea000cf9d3b9d9f09019bf0`다. host 실행의 aggregate source 불변과 input desktop 불변 검사는 결과 `passed: true`에 포함된다. 새로운 환경 block 생성 정책, Windows pseudoentry 처리의 제품 수정, 실제 다운로드/EJS 호환성은 이 작업에서 검증하지 않았다. Rust/Cargo/설정은 수정하지 않았다.
