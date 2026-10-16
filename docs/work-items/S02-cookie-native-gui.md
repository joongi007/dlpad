# S02: 세션 쿠키 네이티브 GUI 합성 검증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R08, R10, R11
- 담당/수정 범위: soak_recovery_audit — scripts/session_cookie_gui_probe.py, crates/engine/tests/session_cookie_fixture.rs, 이 문서, 전용 신규 evidence
- 선행 작업: 세션 쿠키 importer/DownloadSession/UI 통합 및 root immutable CPU release READY

## 사용자 결과

이번 작업에만 선택한 Netscape 쿠키 파일을 네이티브 화면에서 사용하고 완료·취소 후 복사본을 정리한다. 잘못된 파일은 고정 오류로 거절하며 원본과 무관 파일을 보존한다.

## 계약과 범위

README → PRODUCT → CONTEXT, R04/R08/R10, DESIGN, ARCHITECTURE, INTEGRATIONS, ADR-0002/0003을 기준으로 한다. 공유 제품/Cargo/schema/기존 GUI scripts를 수정하지 않는다. 새 실행 파일 빌드가 끝나기 전 앱은 실행하지 않는다. 실제 계정·인터넷 영상·사용자 Documents를 사용하지 않는다. 기존 accessibility lookup/native private Xvfb/DBus 도구는 읽기 전용 재사용한다.

## 인수 조건

- 합성 유효 쿠키의 version에는 --cookies가 없고 metadata/download는 동일한 private literal copy를 사용한다. artifact를 검증한다.
- 실제 작업 취소 후 worker/동일 앱 guardian/unit/cgroup 및 쿠키 복사본 정리를 확인한다.
- 잘못된 source는 도구/job 생성 전 거절하고 고정 정제 오류를 실제 진단 preview에서 확인한다.
- 세 경우 모두 COOKIE_SOURCE AT-SPI Text 값이 소비 후 비워진다. DB/journal/앱 로그/진단 preview에 합성 secret와 source/private copy 경로가 없다.
- 원본 파일과 독립 sentinel identity/content를 보존한다. 사용자 화면을 사용하지 않고 기존 unit/8h soak를 건드리지 않는다.

## 검증과 handoff

Root의 전체 7명령 exit0 후 준비된 `/tmp/dlpad-cookie-app-d_efdw2c/dlpad-cpu` (SHA256 `5cb52a14d83fd1531754b081b821421921afb5fcef58c6cf2afc5746037ae5f8`, 12,475,256 bytes)만 실행했다. 새 라이브러리/런타임 의존성은 없다.

실제 최종 명령:

```sh
python3 -B scripts/session_cookie_gui_probe.py --binary /tmp/dlpad-cookie-app-d_efdw2c/dlpad-cpu --expected-sha 5cb52a14d83fd1531754b081b821421921afb5fcef58c6cf2afc5746037ae5f8 --output /mnt/d/project/dlpad/docs/validation/evidence/session-cookie-native-gui-20261004-accepted
```

exit0, success/cancel/malformed 모두 PASS. [최종 원시 결과](../validation/evidence/session-cookie-native-gui-20261004-accepted/result.json)에서 동일 바이너리 hash, artifact, 소유 unit/cgroup GC, 복사본 제거, COOKIE_SOURCE Entry 값 비움, 원본·sentinel 보존, DB/앱 로그 비노출과 잘못된 파일의 고정 진단 preview를 확인한다. 취소는 실제 AT-SPI `Action.DoAction(0)`으로 수행했다. 원시 쿠키 내용과 private copy 경로는 evidence에 저장하지 않고 동일성·비노출 boolean을 기록한다.

최초 `session-cookie-native-gui-20261004` 및 `-debug` 실패는 새 harness의 click_at import 누락이다. `-final`/`-observed`의 pointer 취소는 AT-SPI bounds top545와 실제 screenshot top525 차이로 Downloading 상태에 머물렀다. `-native-action`에서는 취소·정리 자체가 성공했으나 같은 이름 Label Text와 Entry Text를 혼동한 조회 assertion이 실패했다. 모든 실패 원시를 보존했으며 마지막 조회는 Entry79/Text61만 사용한다. 인수 조건을 완화하지 않았다. Pointer bounds 차이는 별도 UI/접근성 후속 조사 대상이며 이번 native action PASS를 pointer 인수로 표현하지 않는다.

Linux 가상 화면 시험이며 Windows/macOS/실제 로그인 인수를 뜻하지 않는다. 브라우저 가져오기·OS credential store·재개시 재선택 전체 GUI는 남는다. 공유 제품/Cargo 파일을 수정하지 않았고 기존 unit/8h soak/사용자 화면을 사용하지 않았다.

## 독립 검토용 재실행

위 accepted evidence의 boolean만으로 DB/artifact를 독립 검토할 수 없다는 root 검토를 반영했다. 동일 immutable5cb52 앱에 output만 `docs/validation/evidence/session-cookie-native-gui-20261004-root-review`로 바꿔 실제 3case를 재실행했다. exit0/all PASS이며 [결과](../validation/evidence/session-cookie-native-gui-20261004-root-review/result.json)를 기준으로 한다. 기존 원시는 변경하지 않았다.

success/cancel의 `jobs.sqlite`는 read-only source에서 최대2MiB/512pages/3초 bounded backup이다. `final-job.json`은 실제 최종 job/snapshot/options이며 state/attempt/revision/artifact 경로·크기를 포함한다. success는 실제 합성 artifact bytes도 `synthetic-artifact.bin`으로 보존하고 크기/SHA를 기록했다. `synthetic-tool-calls.json`에는 stage/합성 private-copy path/literal byte수와 SHA만 남기고 cookie 값은 제외했다. 원본과 sentinel은 dev/inode/byte수/SHA before-after, fake tool·고정 cookie fixture는 SHA를 기록했다. Root가 이 원시 DB/history/artifact/privacy 근거를 독립 대조한다.

## 최신 회복 gate와 실제 기록 fixture

Root가 전체 Linux7/WindowsGNU3 검증 후 제공한 immutable29ce8c 앱으로 기존 GUI3와 unsafe-cache startup을 새 `session-cookie-recovery-native-gui` evidence에서 반복한다. unsafe-cache는 소유 tmp foreign-cache의 sentinel을 가리키는 symlink이며 고정 오류 label/시작 비활성화/실제 접근성 action 후 no tool/no job/외부 fixture 불변을 인수한다. 공유 앱과 기존 cache를 변경하지 않는다.

새 독점 ignored fixture driver `session_cookie_fixture.rs`는 명시한 owned `/tmp` root의 source/cache만 사용하여 실제 import/defer_cleanup record를 생성한다. symlink/UID/0700/root mismatch를 거절하고 raw cookie 값을 출력하지 않는다. Root가 build·immutable fixture binary를 제공한 뒤에만 same-boot deferred 시작 UI를 시험한다. 현재 same-boot GUI 회복은 미검증이며 cleanup으로 표현하지 않는다. 새 의존성 없이 기존 libc/tempfile/serde를 사용한다.

immutable29ce8c (12,512,376 bytes)의 [GUI3 원시](../validation/evidence/session-cookie-recovery-native-gui/result.json)는 success/cancel/malformed 모두 PASS이다. unsafe-cache 최초는 외부 디렉터리의 직접 경로와 symlink를 통한 sentinel 경로의 문자열 차이를 harness가 혼동한 assertion만 실패했다. 실제 고정 label/시작 비활성화/DoAction 후 no-tool/no-job/source·sentinel 해시 보존은 관측됐다. 표현을 교정한 [unsafe-cache 실제 재실행](../validation/evidence/session-cookie-recovery-native-gui-unsafe-corrected/result.json)은 exit0/PASS다. 최초 원시는 유지한다.

Same-boot 후속 인수는 fixture 실제 record/copy identity/hash 보존, 실제 보존 안내 label과 enabled start, 새 source를 선택한 새 copy의 정상 다운로드와 정리, 이전 deferred copy와 두 record의 불변이다. 기존 deferred 정리 성공으로 표현하지 않는다.

Root가 제공한 immutable `/tmp/dlpad-cookie-fixture-0tc82end/cookie-fixture` (SHA256 `31f3e3ea2fdbededac902a86f085d847cdfcf1226ca7154815d7cd73af259508`, 40,204,720 bytes, 테스트 전용 비배포)를 `--fixture-binary`/`--fixture-sha`로 지정하고 위 immutable29ce8c 앱에 `--case same-boot`를 실제 실행했다. 첫 [same-boot 원시](../validation/evidence/session-cookie-recovery-native-gui-same-boot/result.json)와 root 독립 검토용 [raw 재실행](../validation/evidence/session-cookie-recovery-native-gui-same-boot-raw/result.json) 모두 exit0/PASS다. Actual pending label/start enabled/새 copy Downloaded와 artifact/newcopy 정리/unit GC/기존 deferred copy+record identity/hash 불변을 확인했다.

Raw evidence는 실제 recovery.json/owner.marker의 bounded8KiB before/after를 각각 보존한다. cookie 값은 복사하지 않았다. 두 record before/after byte 동일, recovery record boot=current boot, final DB integrity_check=ok/state Downloaded, 합성 artifact의 실제bytes SHA=recorded SHA를 자체 추가 대조했다. 앱과 fixture immutable SHA before/after도 유지했다. Root의 이후 post-ready 오류 분류 코드가 이29ce8c 앱에 포함되었다는 뜻은 아니다. 실제 로그인/다른 OS/이전 부팅 회복 전체GUI는 여전히 미검증이다.
