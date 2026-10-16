# ADR-0003: 도구 소유권과 선택적 업로드

상태: 소유권 분리·선택 설치는 Accepted(사용자 요구), 세부 구현과 P2 순서는 Proposed.

결정: 사용자 설치를 재사용하고 앱이 설치한 경로만 앱에서 관리·제거한다. 설치/업데이트 manifest로 버전·해시·라이선스를 추적한다. 미디어는 제거 기본 대상이 아니다.

업로드는 다운로드와 독립 job으로 모델링한다. 설정 연결 후 명시적 opt-in, 기본 private, 실패 시 로컬 artifact 보존. OAuth와 resumable upload를 사용한다.

결과: 가장 작은 기본 앱과 선택 도구 크기를 별도·합계로 공개한다. YouTube JS runtime과 FFmpeg 비용을 숨기지 않는다. 모든 OS 제거 방식에서 대화상자를 띄울 수 없으므로 앱 내 정리 UI도 필요하다.

## 2026-10-01 외부 FFmpeg 연결

기존 FFmpeg 파일을 네이티브 입력에서 명시적으로 선택하고 typed argv에 단일 경로로 유지한다. worker의 버전 확인과 Attempt component 저장, 실패 보고서 버전 전달을 연결했다. PATH 밖 임시 복사본으로 합성 빠른/정확 구간 저장과 decode를 시험했다. 외부 파일은 관리 설치로 바꾸거나 삭제하지 않는다. 관리 설치 공급처/서명/라이선스 결정과 다른 OS 실행은 미완료이며 Proposed 범위는 유지한다.

## 관리 직접 실행 파일 후속 개발

직접 yt-dlp의 고정 HTTPS source/size/SHA256/license와 소유 staging, argv-only guardian 버전 검사, 실행 후 digest 재검증, create-only 공개와 active DB 전환을 구현했다. Linux 실제 공식 파일과 변조/취소/DB 실패 fixture를 [실증](../validation/managed-tools-validation.md)했다. 목록/기존 버전 선택은 별도 metadata 조회와 stream digest 검증을 구분하며 active 제거를 거절한다. [네이티브 관리 UI](../work-items/S04-managed-tools-ui.md)는 다운로드에 적용할 때 active를 다시 검증한다. Linux 설치 staging은 PID/start/boot와 파일 identity·checkpoint digest가 일치하는 종료 owner의 잔여물만 회복하며, 미확인 파일은 보존하고 설치 수·용량을 제한한다. 관리 UI 갱신 worker에 회복과 보존 개수 안내를 연결했다. Archive 공급·JS runtime·비Linux staging 회복·final publication crash·전체 GUI/배포 license 검수는 남으며 전체 공급 정책의 Proposed 상태는 유지한다.

## 2026-10-04 Linux cookie journal anchor 구현 근거

실제 anchored 파일·lock·부분 쓰기·reopen 실패 시험을 근거로 별도 recordstate.json의 native directory/record anchor를 recovery record2에 저장한다. marker digest에 anchor를 포함하며 기존 record1은 journal 없이 계속 읽는다. 실제 owner 전체 entries는 cookie/recovery/marker와 journal의 정확 owned_names만 허용하고 unused receipt도 외부 추가로 거절한다. cleanup은 pending slot 또는 실제 proof/identity 불일치에 파일을 보존하며 fixed owned names만 제거한다. 부분 unlink는 원자적이라고 주장하지 않는다.

제품 경로의28개 쿠키 시험으로 import·정상 정리·prior-boot 회수·anchor 교체/unused receipt 보존·v1 호환을 확인했다. 이는 durable anchor 연결에 대한 근거이며, 제안된 I→bind→B→R launcher 및 same-boot receipt recovery 전체를 채택·완료한 근거는 아니다. 같은 boot 보존 정책을 계속 유지한다. 추가 파일1개와 최대 두 receipt, record4096B/receipt256B이며 새 dependency/thread는 추가하지 않는다. 기존 private record lock을 유지하고 긴 I/O는 기존 worker에서 수행한다.

## 2026-10-04 Linux gated reader container의 실험 기반 채택

동일 service의 AddRef와 RemainAfterExit만으로는 종료 후 원래 cgroup events/procs를 다시 열 수 없었다. ENODEV/ENOENT를 빈 그룹 증거로 완화하지 않는다. [전용 slice 실제 lifetime 실험](../work-items/S03-cookie-dedicated-slice-lifetime-probe.md)과 root 독립 재실행에서 parent native identity 유지, 실행 중 populated1 거절, 종료 후 held/new-openat 모두 populated0+빈procs, 정확 ownedunit GC를 검증했다. 이 Linux 컨테이너 수명 방식을 scoped 구현 기준으로 채택한다. 기존 최종 3 OS 인증/UI 선택의 Proposed 상태를 확정하지 않는다.

각 reader에 service와 전용 slice 두 transient unit을 만들며 기존 launcher/guardian과 동적 libsystemd를 재사용한다. daemon·dependency·UI server는 추가하지 않는다. 추가 journal mutex는 최대4KiB record/256B receipt 검증·sync 동안만 잡고 긴 tool 실행에서 놓는다. FD3개만 launcher에 상속하고 writer/record lock을 guardian/tool에 전달하지 않는다. 각 worker의 새 lease는 metadata0/download1 두 슬롯을 한 번씩 사용하며 재개에는 새 인증 선택을 요구한다.

제품 process/download 연결은 통합 컴파일와 engine library244PASS/2ignored를 확인했다. 직접 launcher native3PASS와 제품 DownloadRepository 수직 인수는 다른 범위다. 부모 release의 foreign sibling/queued start 경합, 초기 identity 실패 자원관리, app crash 뒤 same-boot startup recovery는 남는다. runtime StopWhenUnneeded 변경 후보는 실제 EROFS로 거절되어 채택하지 않았다.

## 2026-10-04 부모 의존성 및 이전 부팅 삭제 권한 보강

runtime setter 대신 service와 StopWhenUnneeded=true auxiliary slice를 단일 transaction으로 생성하는 방식을 채택한다. 독립 실제 실험은 정상·live sibling·queued sibling 3개와 exit7/signal15 두 개를 통과했다. 제품에서는 gated service만 RemainAfterExit/ignore-failure를 쓰고 원 ExecMainCode/Status를 성공 판단에 사용한다. 관리 Result=success를 도구 성공으로 해석하지 않는다. 원 native 빈 그룹과 receipt sync 뒤 exact service Stop/Unref 및 parent Unref만 수행하여 부모 직접 Stop의 관측 이후 경합을 제거한다. 실제 제품 gate3개·다운로드4개·sameboot 회수2개를 재검증했다. 실패 원시와 수정 근거는 [통합 작업](../work-items/S02-cookie-service-dependency-integration.md)에 보존한다. 초기 identity 미확보 및 제품 foreign queued fixture는 아직 별도 인수다.

이전 부팅 Intent/Bound 회수에는 [PreviousBootAuthority](../work-items/S02-cookie-priorboot-pending-recovery.md)를 별도로 사용한다. 실제 kernel UUID와 원 record/marker boot·digest 및 native ownership을 검증하며 모든 삭제 전 권한을 재검증한다. 정상 및 sameboot eligibility를 완화하거나 receipt/Resolved를 만들지 않는다. 테스트 boot 주입은 cfg(test)에만 있다. 실제 lease의 네 조합 회수와 engine library251개를 검증했지만 실제 kernel 재부팅 인수는 아니다. 새 schema·dependency·상주 thread는 없으며 bounded kernel boot read와 기존 최대 두 slot의 검증 비용만 추가한다.


## 2026-10-04 EOF keeper로 초기 identity 실패 정리 보강

앞 절의 gated RemainAfterExit/ignore-failure 방식은 초기 Invocation 확보 전에 active(exited) 서비스가 남는 문제로 대체한다. reader는 RemainAfterExit=false와 실제 원 종료값을 유지하며 parent 밖 app.slice의 별도 EOF keeper가 parent 수명을 유지한다. keeper에는 고정 Rust helper marker와 read-only pipe만 전달하며 writer는 launcher만 소유한다. reader·keeper·parent는 하나의 transaction으로 생성한다. 시작 Invocation의 빈 값은 제한된 시작 관측에서만 기다리고, 이후 소유권과 종료 확인은 원 Invocation/native identity/pidfd를 엄격히 검증한다. Delegate는 실제 DBus boolean으로 확인한다.

원 reader 종료·parent held/fresh empty·receipt sync를 확인한 뒤 writer EOF, keeper 원 종료 확인, 정확한 소유 서비스 정리, parent Unref 순으로 종료한다. 부모 직접 Stop이나 제품 RuntimeMax를 추가하지 않는다. 추가 비용은 인증 reader당 helper 하나·service 하나·pipe FD 두 개이며 새 dependency/schema는 없다. RSS·성능 인수는 별도다.

[실제 제품 초기 실패 검증](../work-items/S02-cookie-keeper-startup-fault-native.md)의 정상 대조와 세 초기 실패 단계가 통과했다. [통합 기록](../work-items/S02-cookie-eof-keeper-integration.md)의 서비스3·다운로드4·same-boot 회수2 시험도 통과했다. [독립 keeper 실험](../work-items/S03-cookie-dependency-holder-probe.md)의 foreign live/pending 보존은 실험 범위이며 제품 foreign fixture와 전체 OS 인수를 대체하지 않는다. 최신 전체 workspace 검증은 별도 실행 중이다.

후속 [제품 sibling 실제 검증](../work-items/S03-cookie-keeper-sibling-native.md)은 receipt 이후 active sibling과 activating/nonzero Job을 투입해 원 Invocation/PID/pidfd/Job 및 parent native identity 보존, 정확한 reader/keeper GC, fixture EOF 뒤 정상 종료와 parent 자동GC를 확인했다. 직접 Stop 호출 계측은 아니며 제품 foreign service를 건드리지 않는 native 동작 증거다. 최신 전체 통합은 [별도 검증](../work-items/S03-cookie-keeper-options-workspace.md)을 따른다.

## 2026-10-04 bounded ZIP 공유 계약 구현

단일 실행 파일 직접 설치와 별도로 ArchivePackage(version1)를 구현한다. archive SHA/bytes와 각 component SHA/bytes를 분리하고 flat allowlist8개/ZIP directory32개/압축 및 expanded 각각256MiB를 제한한다. Windows device name·case alias·unknown schema·외부 ownership는 거절한다. rawzip0.5.1의 caller-owned buffer iterator와 flate2 1.1.10 rust_backend만 추가하며 다른 ZIP codec/암호화/임의extract-all은 제공하지 않는다. 기존 flate2 lock package를 재사용하고 rawzip 하나가 lock에 추가됐다. 공급 trust/license·활성화·설치 crash복구·OS별 제거와 GUI는 별도이며 API 구현만으로 공식 관리 공급의 Proposed 결정을 Accepted로 바꾸지 않는다. 검증 기준은 [작업 계약](../work-items/S04-bounded-archive-contract.md)이며 실제 측정 후 비용을 기록한다.

## 비활성 archive 세트 통합

archive 설치는 사용자 지정 원본을 읽기 전용으로 검증하고 별도 비활성 세트를 게시한다. 목록 metadata 조회와 선택/사용/제거의 전체 hash 검증을 구분한다. 설치·active 선택·제거는 동일 root lock을 사용하며 원 root 및 owned file/dir native identity를 보유한다. Windows 제거는 원 DELETE capability, Unix는 원 directory fd unlinkat를 사용하되 부분 삭제/동일 사용자 namespace 경합을 원자 rollback으로 주장하지 않는다. staging v2 최대10개 증거와 기존 v1을 병행한다. HTTPS 공급·publisher trust/license/notice·runtime version·GUI·final publication crash 회복은 별도 게이트이며 공급 정책의 Proposed 상태를 유지한다. [통합 기록](../work-items/S04-package-native-boundaries.md) 및 [후속 HTTPS 계약](../work-items/S04-archive-https-worker.md)에 검증 범위를 기록한다.

## 2026-10-04 외부 Deno 선택 연결

외부 Deno는 optional typed 경로로 선택하며 원 사용자 옵션 snapshot을 유지한다. 실행 worker가 canonical 원 file/native identity·hash와 실제 stable host version을 확인하고 metadata/download에 동일 literal argv를 전달한다. runtime flags를 raw 고급 옵션으로 덮어쓰지 않는다. 버전은 기존 Attempt component와 정제 진단에 기록하고 외부 파일의 소유권이나 공급 신뢰를 변경하지 않는다. 네이티브 입력의 빈 값은 기존 yt-dlp 기본 runtime 정책이며 선택하면 해당 Deno만 사용한다. 재개는 현재 편집값 대신 저장된 원 snapshot을 사용한다. 실제 GUI 및 OS별 인수 전 공급·최종 스택의 Proposed 상태를 유지한다. [작업 계약](../work-items/S04-external-deno-native-input.md).

## FFmpeg TAR 기반 추가의 범위

현재고정Linux FFmpeg원공급은GNU tar.xz이며선택ffmpeg/ffprobe/notice가271.65MiB로기존ZIP256MiB계약을초과한다. 기존ArchivePackage상한을조용히변경하지않는다. 별도bounded uncompressed TAR reader는원member→flatcomponent명시mapping/원root/4096entries/768MiBdeclared/384MiBselected/800MiBstream의최대제약으로구현했다. caller가압축원SHA/nativepin/XZ메모리/빈출력File capability와publication·부분실패cleanup을담당한다. GNU고정magic과zeroreserved만명시허용하며longname/PAX/sparse/link/special을거절한다. source digest검증을하지않는reader를verifiedarchive로표현하지않는다. [계약과현재회귀](../work-items/S04-bounded-tar-reader.md). 실제공급전체추출/관리installer·활성화·용량예약/라이선스와3OS인수는미완료다.

## 제한된 XZ 공급 판독기

lzma-rust2 0.21.0을defaultfeatures없이std/xz만고정하고per-blockLZMA2 memory64MiB를설정했다. encoder/lzip/optimization/C라이브러리는추가하지않는다. sha2 0.11계열포함lockpackage8개가추가되었으며독립검증예제486280B/actualRSS10348KiB를측정했다. 프로세스전체RSS64MiB보장으로해석하지않는다. 원File정확SHA/size전후검증·singleXZ정확compressed소비·TAR전체종료확인으로component를읽는다. 실제공식LinuxFFmpeg세component추출SHA일치는통과했고현재3회전체순회비용및관리installer/선택/라이선스공급gate는남는다. [계약과실측](../work-items/S04-bounded-tar-xz-gate.md).

## TAR/XZ 명시 v2 관리 설치

ArchivePackage v1 layoutNone는기존ZIP256MiB와JSON필드생략을보존한다. v2는TarXz root+member→component매핑만받고선택합384MiB를검증한다. 설치는원sourceFile/nativeidentity와rootlock/동일120초deadline/staging512MiB에연결하고임의format추측/전체메모리/다른삭제fallback을추가하지않는다. 단일component추출을세번검증하는현재설치는실제고정FFmpeg비활성전체설치/제거/원SHA보존PASS,설치36682ms였다. 검증예제전체3788824B/최대RSS12808KiB이며이는앱증분크기가아니다. 대응source/recipe/notice/trust·정확runtime선택·GUI/3OS공급은별도미완료다. [공유계약과실증](../work-items/S04-tar-xz-package-integration-contract.md).

## 관리 실행lease 기반의 제한된 검증

기존rootmutationlock과sharedExecutionLease를동일원lockfile에사용한다. 새의존성/schema/상주worker는없고파일descriptor하나와공유잠금비용을추가한다. guardDrop은close-only로Unixduplicatedreader가남았을때전체open-file-descriptionunlock을피한다. Linux실제공유reader경합·lastrelease·외부symlink보존·parentdrop/duplicate유지3회귀PASS. guardian/사용자서비스capability전달과WindowsLockFileEx수명,앱crash/TerminalUnconfirmed삭제보류는별도[제안계약](../work-items/S04-managed-execution-lifetime.md)이며아직채택/제품완료로표현하지않는다.

## durable 관리 실행 보류의 첫 통합

sharedlock만으로appcrash/미확인reader보호를완료할수없어create-onlyboundedpendingmarker를mutation권한에연결한다. 기록은원root/filepin·nonce/원bytes로검증하고명시finish뒤만clear하며Drop으로삭제하지않는다. 원options/resume관리pathworker에연결했고Linuxmarker/lease8회귀·CPUentry81·CPUworkspaceClippy를통과했다. 새dependency/schema상주worker없이작은file·sync와잠금비용을추가한다. 자동회수/actualguardiancrash·Windowsnative인수는별도이며Unixnamespace경합·Windowsdirectoryflush한계를숨기지않는다. [통합계약](../work-items/S04-managed-execution-pending-integration.md).

## 2026-10-05 별도 고정 notice 공급 계약

공식 runtime ZIP에 executable만 있는 원 자료를 확인했으므로 notice를 존재하지 않는 archive member로 선언하지 않는다. v3 flat ZIP에 Notice만 별도HTTPS/revision/기존component bytes/SHA로 참조하는 계약을 추가한다. 기존 v1/v2 serialized bytes와 layout은 유지하고 새로운 dependencies는 없다. 실행 공급과 notice에 원 handle/native identity·bounded64KiB stream·하나의120초 기한·하나의 install lock·소유 stage journal 및 전체 set 검증을 적용한다. 실패 시 기존 active와 외부 원자료를 보존한다. 이 세부 계약은 구현 선택이며 현재 native 인수 전이다. 전체 도구 공급/법적완비 또는 Proposed architecture 채택을 뜻하지 않는다. [범위와 인수](../work-items/S04-external-notice-package-transaction.md).
