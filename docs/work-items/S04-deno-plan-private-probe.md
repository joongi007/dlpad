# S04: Deno 선택 계획의 private GUI 수집기

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R05, R06, R08, R13
- 담당/수정 범위: diagnostic_cache; scripts/cpu_deno_plan_probe.py, 이 문서만
- 선행 작업: Deno CPU 입력 통합, root의 frozen release SHA 인수

## 사용자 결과

선택 Deno 절대 경로가 실행 계획의 단일 argv 값에 그대로 보존되고 빈 입력과 상대 경로가 각각 기본 탐색과 고정 오류로 처리되는지를 실제 네이티브 GUI에서 관찰한다.

## 계약과 범위

기존 cpu_plan_probe.py의 Xvfb/private D-Bus/AT-SPI/임시 데이터/소유 자식 정리를 재사용한다. Rust, Cargo, 기존 scripts를 수정하지 않는다. 합성 파일과 경로만 사용하며 도구 실행, 네트워크, 사용자 clipboard, desktop fallback은 없다. SHA 일치 전 프로세스를 시작하지 않고 종료 후에도 확인한다. 새 evidence 디렉터리만 생성한다.

## 인수 조건

- Deno literal에 공백, 달러, 작은따옴표를 포함해 field 실제 Text 값을 확인한다.
- GUI 계획에서 --no-js-runtimes와 --js-runtimes가 각각 1회이고 다음 값은 deno:<원 경로>와 같다.
- 빈 입력에서는 두 flag가 없고, relative 입력은 고정 오류이며 이전 계획을 노출하지 않는다.
- 원본 URL은 생략되고 jobs.sqlite/실행 sentinel이 생성되지 않는다.
- 관측 노드 256, scroll 16회, 대기 15초, 로그 기존 bounded_log 상한, 결과 64KiB로 제한한다. 소유 PID 및 Xvfb 정리를 기록한다.

## 검증과 handoff

Python AST parse PASS, `--validate-only` exit 0 (arguments valid; no processes started). 실제 GUI 실행 전 상태는 수집기 구현 완료 / 검증 대기이다. 실제 GUI 실행은 root 최신 release와 SHA 전달 후 통합 담당이 수행한다. 실제 GUI, 실제 runtime 동작, Windows/macOS는 아직 미검증이다. 계획은 선택 경로의 버전/실행 호환성을 검증하지 않는다.

### 첫 실제 실행의 실패 원인과 교정

`docs/validation/evidence/deno-native-input-linux-gui-first/result.json`은 exit 1, 선택 literal/argv 일치, 이후 deadline 초과, cleanup true, SHA 전후 동일이다. 원시 파일을 변경하지 않았다. failure.png에는 Deno 원 문자열 전체가 선택된 채 남아 있다. 재사용 key_input은 Ctrl+A 후 전달 문자를 입력하지만 빈 문자열일 때 삭제 키를 보내지 않는다(scripts/subtitle_ui_probe.py:73–85). 따라서 빈 입력 시험은 실제로 빈 입력이 아니었고 flag 소멸 관측이 끝나지 않았다. 신규 수집기만 빈 값에는 BackSpace를 추가하고 모든 field 변경 후 fresh AT-SPI Text exact 값을 확인하도록 교정했다. 각 selected/empty/relative/restored 단계는 실제 header GetState Expandable/Expanded를 읽고 collapsed일 때만 펼치며 fresh 상태를 기록한다. timeout 상한은 늘리지 않았다. 제품 결함으로 단정하지 않는다. 교정 후 실제 GUI는 root 재실행 대기이다.

第三 실행 visible-plan-fix는 같은 이름의 caption을 본문으로 오인하여 prefix `실행 계획 내용` (20 bytes)를 JSON 파싱했고 실패했다. 새 수집기는 named node의 fresh GetRole을 먼저 확인하며 정확한 LABEL(28)만 건너뛰고 동일 이름의 TEXT(61)/ENTRY(79)를 탐색한다. 예상 외 role 또는 nonempty malformed JSON은 진단 후 실패한다. 빈 본문과 상대 입력 정책, bounded traversal/scroll는 유지한다. 이전 raw와 제품 Rust는 보존했다. AST PASS; 실제 재실행 대기.

role-fix 실행은 actual GetRole `(uint32 29,)`를 기록했다. 앞선 LABEL=28 상수는 잘못 추측한 값이었다. `docs/validation/evidence/deno-native-input-linux-gui-role-fix/result.json`의 실제 label role29와 root UI caption 생성 근거에 따라 정확29만 skip하도록 수정했다. ENTRY79 입력 관측은 유지하고 timeout은 변경하지 않았다. 원 role-fix 실패 raw 보존.

## 실제 Linux GUI 인수 완료 범위

`cpu_deno_plan_probe.py`의 고정SHA observed-role 재시험은 terminal exit0/PASS이다. root가 result.json의 실제Text·selected/empty argv·relative고정오류·원URL생략·DB/실행sentinel미생성·closeexit0/ownedcleanuptrue/SHA전후일치를직접대조하고 relative.png의오류가시성을검토했다. [원시 결과](../validation/evidence/deno-native-input-linux-gui-observed-role/result.json). 사용자인증/영상/clipboard/visiblefallback은없다. 신규화면입력은공백·달러·작은따옴표의literal이며한글path는별도core/UI단위회귀다. 실제한글IME·WindowsGUI Deno/공식runtime·macOS·관리공급전체는후속게이트다. 첫네실패원시는보존한다. 최신Linux기본/CPU전체와WindowsCPU전체도terminalPASS/source현재SHA일치이며전체P0완료를뜻하지않는다.
