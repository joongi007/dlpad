# S00: 현재 설치 후보 CPU의 격리 GUI 비용·한글 입력 인수

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R01, R03, R08, R11
- 담당/수정 범위: root — 현재 artifact 고정·scripts/windows_current_install_gui_cost_probe.py·native 직렬 실행·통합·검증·문서; metrics_recovery — /tmp 현재 CPU 비용 collector 초안; diagnostic_cache — /tmp 기존 Windows IME 증거와 소유 private 화면에서 가능한 OS composition 검사 감사; soak_recovery_audit — /tmp 증거 독립검토
- 선행 작업: 현재 Windows 전체4검사/실제 copied CLI 인수. 개발 ZIP 작업의 source/artifact pin을 유지하고 비용 측정은 다른 native 작업 종료 후 실행한다.

## 사용자 결과

이전 watch500 snapshot 성능/한글 결과를 새 설치 후보 CPU에 소급하지 않고 현재 고정 binary로 확인한다. 격리 화면으로 원 user desktop/focus/clipboard/실영상과 Documents를 건드리지 않는다.

## 계약과 범위

CPU f3bae801…/7,917,568B의 현재 Windows GNU proof와 소유 합성 TEMP roots를 사용한다. 비용은 기존 엄격10초CPU<1%/working set<80MiB 구간3회·각60초를 재검토하며 실제 입력·목록 반영 인수와 구별한다. 기존 source/root/native capability·Job/natural terminal·private desktop/input desktop guard를 유지한다. 한글은 기존 Unicode PostMessage/readback 및 실제 IME 경로의 차이를 먼저 감사하고, Microsoft API 근거와 제한을 기록한 뒤 독립적으로 가능한 private OS composition 시험을 구현한다. visible fallback/global SendInput/사용자 input language 변경/실제 계정 사용 없음. root가 신규 helper/shared변경을 조정한다.

## 인수 조건

고정 CPU/collector/helper/host/source pins와 실제 원log/counters·60초3회18구간 CPU/WS 판정, 실패 보존. IME는 한글 문자주입·조합/확정/취소·OS native 경로·물리 키보드/reader를 분리해서 보고한다. 불가능한 private API면 그 시험만 보류하고 다른 P0를 계속한다. 전체 스택/IME 인수를 근거 없이 완료 선언하지 않는다.

## 검증과 handoff

현재 독립 collector/IME 감사 초안 단계. native GUI 실행은 root만 직렬로 조정한다. 이번 변경은 benchmark/test 도구이며 제품 dependency/lock 변경 없음.

현재 CPU 고정 비용 collector를 root가 native private 화면에서 직렬 실행 중이다. IME 감사는 기존 owner IMM fixture 경로를 재사용할 수 있음을 확인했지만 현재 실행/물리 입력 인수는 아직 아니다. 감사의 R09 표기는 업로드 요구와 다르므로 이 workitem에서 GUI 인수·Windows 검증에 해당하는 R08/R11로 정정한다.

실제 비용 underlying native3PASS/18창을 root가 원수식·pins·Job0·private/input/MTA로 대조했다. 원 wrapper exit1/FAIL은 성공 경로의 absent owned_job_terminated를 false로 강제한 wrapper계약 오류로 보존했다. explicit outer forcedFalse와 원 naturalempty/active0를 함께 확인하는 v2를 scripts에 통합했으며 신규 native 재실행 성공으로 표현하지 않는다. CPU 평균0.416534/0.286333/0.338306%, 최대10초0.781303/0.618860/0.624280%, peakWS76,533,760/71,561,216/70,852,608B. root가 round1 ready PNG에서 한글label/합성파일 행을 직접 확인했다. 전fixture/DB 독립검토 및 현재Unicode·같은source ownerIMM fixture 인수는 진행 중이다. Thread set churn으로 thread operation attribution은 미검증이며 wholeJob예산과 구별한다.

현재 defaultCPU Unicode 실제 `dlpad-current-ime-native-run.py --case unicode --binary …/inputs/cpu.exe --binary-sha f3bae801…` exit0/nativePASS. source570/pins·47단계focus/geometry·exact 한글 입력 123 readback·원sentinelbytes·자연exit0/Job0/private/MTA/input을 root/독립 대조했다. root는 hangul-input.png를 직접 확인했다. 최초 root rawlog strictUTF8는 native mixed encoding으로 DecodeError였고 원bytes의 ASCII 종료guard로 재검토했다. 실제IME·물리keyboard·reader·OS200%DPI는 미검증이다. [현재 문자 인수](../validation/evidence/windows-current-install-unicode-20261006-continue/completion-review.json).

root는 round3 idle-end PNG도 직접 확인했고 한글label·로컬합성file rows 표시를 관측했다. 같은source defaultOFF ownerIMM feature의 clippy/test/build를 root소유 nativehost에서 직렬 실행 중이다. 기존 immutabledefaultCPU를 보존하고 이번 build가 생성한 정확featureSHA만 확인해 원default를 복원하며 각각proof를 구별한다.

30000개payload exactname/3B와3DB각10000행·integrity/정확path의 실제 독립검토도 PASS로 통합했다. [현재비용완료범위](../validation/evidence/windows-current-install-artifact-cost-20261006-continue/completion-review.json). WSL NTFS metadata읽기로 약수분 지연됐으며 진행중 근거를확인했다. 다음collector는scandir/원진행기록 최적화를후속으로 남긴다. 실제nativeID재조회/quiet host/8hGUI 또는 threadoperation인수로 확대하지 않는다.


완료 범위: 현재 defaultCPU f3bae801… 비용3×60/18창·actual30000payload/3DB·Unicode47단계와 같은source feature58ea9ed9… ownerIMM cancel/commit2case를 root/독립 검증했다. feature의 `cargo clippy -p dlpad --all-targets --locked --features cpu,ime-compose-fixture -- -D warnings`, `cargo test -p dlpad --locked --features cpu,ime-compose-fixture`, `cargo build -p dlpad --bin dlpad-cpu --release --locked --features cpu,ime-compose-fixture` 모두exit0/private naturalguards PASS. 해당testvariant7,950,336B(+32,768B)와default7,917,568B를구별하고 원defaultSHA복원을확인했다. root가preedit한글/취소뒤빈field/확정뒤한글의PNG를직접확인했다. [원빌드](../validation/evidence/windows-current-ime-feature-build-20261006-continue/root-review.json), [취소](../validation/evidence/windows-current-install-ime-cancel-20261006-continue/completion-review.json), [확정](../validation/evidence/windows-current-install-ime-commit-20261006-continue/completion-review.json).

이는 defaultbinaryOScomposition·physical자모/candidate·실제reader/clipboard·cleanOS/DPI·8hGUI 또는 최종스택채택/P0전체 완료가 아니다. defaultOFF기존fixture를 재사용했고 제품Rust/dependency/lock 변경0. 원wrapperFAIL/실제nativePASS·최초rawencoding오류를 보존했다. 다음 [Windows 명시 설치 선택 기반](S07-windows-active-install-coordinator.md)을 이어간다.
