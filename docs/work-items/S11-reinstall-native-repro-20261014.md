# S11: 제거 후 동일 위치 재설치 native 재현

- 상태: completed
- 분류: bug
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: installer_batch; 본 문서와 .local-tools/session-recovery-20261013/reinstall-native-repro-v1 신규 helper만
- 선행 작업: frozen v24 whole/source670/current3/payload7 Root 검토

## 사용자 결과

앱 제거 후 동일 위치 v24 재설치 MISSING 오류를 소유 임시 설치에서 재현하여 실제 단계와 target/journal 존부를 보고한다. 제품 수정은 Root 담당이다.

## 계약과 범위

ADR0005 Proposed/ADR0009 Accepted 경계를 유지한다. UUID tmp 제품 GUI Install/Verify/Remove/Install만 사용한다. 보존 journal을 별도로 삭제하지 않는다. 사용자 Desktop/실제 설치/외부 자료·제품/Cargo/schema/큰 build target/가시 desktop 변경 제외. Root source671 추가 후 원670 proof는 현재Live 성공으로 승격하지 않고 frozen3PE 재현으로 구분한다. 현재 Live before/after는 별도 검사한다.

## 인수 조건

원7 공급/5payload hashes·현재Live/current3 before-after·원PID/creation/privateDesktop/MTA/Job/sentinel 보존. 제거 확인과 target 부재/journal 유지 관측. 재설치 readiness/최종 catalog MISSING/target/journal/default command 관측. worker terminal 이후만 WM_CLOSE, 실제 exit/자연 Job0 측정. 재현 수집0은 제품 성공이 아니다.

## 검증과 handoff

Python AST PASS; C# compile/native 아직 미실행. exact 명령/diff/basepins는 helper HANDOFF에 기록. 원 v24 helper/E 불변. 실제 실행은 Root source freeze 일정 조정 후 수행한다.

### 원 v1 실제 결과 보존

C# compile은 exit0, private native collection은 exit1로 종료했다. 설치→Verify→제거는 성공했고 실제 bundle 부재·원 journal 5파일 유지가 관측됐다. 이후 Install201 disabled, Verify203 enabled/default203이어서 Install-ready assertion이 실패했다. 원 happy helper의 의도적 readonly journal 표시 변조가 남아 있던 혼입도 확인했다. 제품 재설치 오류 원인이나 MISSING 재현으로 단정하지 않는다. 원 FAIL/cleanup/로그는 `windows-msvc-ui-feedback-v24-reinstall-private-v1-20261014`에 보존했다.

v2 새 namespace는 표시 변조를 제외하고 실제 활성 Install201 또는 Verify203만 요청하며 요청 operation을 구분한다. Python AST PASS, Root source freeze 완료 통지 대기 중이며 아직 v2 native 실행0이다.

### v2 실제 MISSING 재현

2026-10-14 fresh v2 C# compile/native collector/outer wrapper 모두 exit0으로 종료했고 진단 데이터 수집이 완료됐다. 제품 재설치 성공은 아니다. 원 v24 source670/PE를 재현했고 현재 Root source671은 별도 before/after 불변이다. 제거 후 bundle은 부재, 원 .bundle-install-journal은 존재했고 원 5기록 SHA가 유지됐다. UI는 Install201 disabled/Verify203 enabled/default203을 선택했으며 실제 요청은 Verify203이다. 그 요청 후 catalog::MISSING 정확 문구가 관측돼 사용자 오류를 재현했다.

원7 공급/3PE와 sentinel 보존, 자식 총수1, installer natural exit0/Job0/forced cleanup false/private desktop 복구 및 사용자 input desktop 불변을 통과했다. 원v1 FAIL은 그대로 유지한다. 증거: `windows-msvc-ui-feedback-v24-reinstall-private-v2-20261014/result.json` 589117바이트 SHA `1b2df99cfcbc651fc982ab589f15a4e7aa5865d036873e9cc3f54e329dd8fd76`. Root가 새 journal generation 정책을 구현·전체빌드·후속 실제 재설치로 검증할 예정이므로 상태는 in-progress로 유지한다.

### v25 새 정책 실제 재설치 인수

새 v3는 source671/whole1542·0·42/current3/generated7 공급을 사용했다. C# compile/native/wrapper 모두 exit0이며 fresh `windows-msvc-ui-feedback-v25-reinstall-private-v3-20261014/result.json`은593076바이트 SHA `0af7dcd770a01810bfa5ad6f04170833c402661559cc15bc0f6f02b399b3b5fb`다. 제거 후 Install201 enabled/default201, UI104 새 .bundle-install-journal-1을 관측했고 같은target 실제 Install→완료/MISSINGfalse를 확인했다. 기존5기록 SHA는 그대로이고 새5기록/최종5payload도검산됐다.

최초installer닫기 natural0 뒤 같은privateDesktop/Job에 fresh noargs installer를시작하고 소유target 명시입력→matching -1 자동선택/default203→Verify성공을확인했다. 양쪽installer natural0/원Job0/총processes2/forcedfalse/inputdesktop불변이다. 앱 Run은 별도이며 수행하지 않았다. 원7파일/현재source/current3 before-after를유지했다. 완료화면2개를직접view_image했고한글완료/기록-1/최종상태·버튼행을확인했다. 물리입력/IME/OSDPI/cleanOS검증을대체하지않는다.

진단raw주의: before_prepare/after_prepare의 journal_directory_exists는 함수에전달된 oldbasejournal을측정하고 journal문자열은 UI104의new -1을읽었다. 두field를같은path로해석하지않는다. 새journal설치전부재는별도실행Require에서확인했으며새5SHA는최종독립검산됐다. 원raw를교정하지않고scope를명시한다. Root의최종독립검토및문서통합전까지상태in-progress유지.

### Root 독립 통합 검토와 범위 마감

[Root 재설치·cold 검증](../validation/evidence/windows-msvc-ui-feedback-v25-reinstall-private-v3-20261014/root-review.json)의 실제 상태는 `PASS_SCOPED_NATIVE_REMOVE_REINSTALL_COLD_VERIFY`다. source671 및 원33핀 재해시, oldjournal5 보존/newjournal5·finalpayload5 재해시, 실제 Install201 재설치/cold matching→Verify203 성공, 두 installer natural exit0/Job0를 확인했다. 실제 앱 실행은0회다. Root가 [제거 직후 화면](../validation/evidence/windows-msvc-ui-feedback-v25-reinstall-private-v3-20261014/gui/after-remove.png), [재설치 완료](../validation/evidence/windows-msvc-ui-feedback-v25-reinstall-private-v3-20261014/gui/after-reinstall.png), [fresh 검증 완료](../validation/evidence/windows-msvc-ui-feedback-v25-reinstall-private-v3-20261014/gui/fresh-installer-verified.png) 원3PNG를 직접 검토했다. 제거 직후 PNG는 admission pending이므로 Install 준비 완료 증거로 사용하지 않는다.

본 작업의 독립 재현과 후속 native 회귀 인수 범위는 완료했다. 원v1 FAIL/v2 MISSING 및 historical source670 재현 범위를 그대로 보존한다. 전체P0 완료, 실제 사용자 입력/IME·물리 GUI·cleanOS·공개릴리스/업그레이드/서명 완료는 주장하지 않는다. Root 검토에서도 physical_or_clean_os_acceptance=false와 release_ready=false를 유지했다. 원basejournal존부 필드의 제한은 위 설명과 Root snapshot_scope_note를 따른다. check_docs는 Root/패키지 담당 검증으로 중복 실행하지 않았다.
