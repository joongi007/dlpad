# S03: Windows 파일 스캔 종료 대기 실제 인수

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R03, R01
- 담당/수정 범위: root는 이 문서·공유 fixture 계약·후속 통합 및 검증을 담당한다. soak_recovery_audit는 /tmp/windows_library_scan_closewait_plan.md 읽기 감사만 담당한다.
- 선행 작업: Windows 10k 목록·자동 변경 시험, 협조적 scan cancellation 및 앱 CloseWait 구현.

## 사용자 결과

파일 스캔 중 중단 후 종료를 선택하면 worker ACK까지 기다리고 기존 성공 목록을 보존하는지 실제 Windows에서 확인한다.

## 계약과 범위

기존 source와 테스트 seam을 먼저 감사한다. 필요 시 별도 기본 OFF fixture를 root가 조정한다. 사용자 화면과 실제 미디어는 사용하지 않는다. 다른 에이전트와 같은 파일을 동시에 수정하지 않는다.

## 인수 조건

실제 scan 진행 지점을 관측하고, 종료 선택 이후 ACK 전 창이 유지되는 것과 scan 완료·중단 ACK 뒤 원 app 자연 종료·Job0·private desktop 정리를 검증한다. 단순 시간 지연이나 큰 폴더의 실행 시간을 scan-inflight 증거로 대신하지 않는다. 기존 DB snapshot의 실패 경로 보존을 독립 확인한다.

## 검증과 handoff

현재 읽기 감사 중이며 실제 Windows scan CloseWait 인수는 미완료다. 보고서 저장 CloseWait 성공은 이 작업의 완료 근거가 아니다.

읽기 감사 결과, panel busy나 큰 폴더만으로 실제 스캔 진행 지점을 확정할 수 없다. 첫 일반 파일의 metadata/native identity 및 임시 observations INSERT 뒤, publication 전의 기본 OFF `library-scan-closewait-fixture` callback을 준비한다. soak_recovery_audit는 `/tmp/dlpad-library-scan-closewait.rs` 초안과 실패 경로 회귀만 담당하며 프로젝트 파일을 직접 수정하지 않는다. root가 기존 source SHA·공유 Cargo feature·앱 원 이벤트 capability 및 collector를 조정하고 통합 검증한다. 기존 library DB byte guard는 유지한다. 아직 feature/API/native 시험의 구현 완료는 아니다.

앱 연결 초안은 diagnostic_cache가 `/tmp/dlpad-library-scan-closewait-fixture.rs`와 `/tmp/dlpad-accessible-library-closewait.rs`만 담당한다. engine 초안과 담당 파일이 겹치지 않는다. 원 started/release/workerACK 이벤트 3개·20초 대기 상한·실제 worker 자원 반환 후 ACK를 유지한다. root는 Cargo/module 초기화 연결 및 프로젝트 게시를 담당한다. 현재 Rust 제품은 변경하지 않았으며 실제 feature 시험은 후속이다.

root는 원source 및 세draft SHA를 대조하고 engine/library.rs·accessible_library.rs·새library_scan_closewait_fixture.rs를 통합했다. app의 CPU/accessible 두entry에 module/초기화를 연결하고 공유 Cargo feature를 기본OFF로 추가했다. 기존windows-sys0.61.2 직접edge를 재사용하며 새dependency/lock변경없음이다. 의미있는engine rollback/timeout/firstonly/empty 네시험, nativecap module다섯시험, actualappworker ACK/원DB보존시험을 준비했다. 실제 native compile/test는 다음 실행 결과를 따르며 아직 완료가 아니다.

첫 feature fmt/Clippy 및 실제전체tests는 exit0였으나 test출력65650B로 rootcollector64KiBtail에서 원header가 밀려 rawpassed=false였다. 소유header보존 수정과 원실패 기록은 S00-native-collector-header.md를 따른다. 수정후 seed를 포함한 재검증의 최초Clippy는 seed의 rusqlite FromSql u64미지원으로 compile101이었다. root가 signed i64읽기 및 checked u64변환으로 수정했고 실제전체4명령을 다시 실행한다. GUI native인수 성공을 미리 주장하지 않는다.

수정후 Windows GNU cpu,library-scan-closewait-fixture 전체fmt/clippy/test/release 네명령이 모두통과했다. evidence/windows-scan-closewait-feature-seed-fixed-20261005-resume/root-review.json에서 현재source SHA·10개 새의미있는회귀·engine316PASS/7ignored·Cargo.lock불변·고정featureexe를 독립 대조했다. ignored시험을실행성공으로표현하지 않는다. actualprivateGUI release/timeout과원DB논리보존은후속이다.

root가 C# 관측기와 bounded Python runner를 통합했다. 실제seed는 인자없이 project TEMP에새protectedSID ACL root/data/media·CAP·sentinel·원DB1행/oldfile·미등록newfile을 만든다. runner는 CAP·원namespace와binary/source SHA를 확인하고 actualfirstobserved/preRelease/alive/workerACK/naturalexit0/Job자연0후에만 boundedarchive를 남긴다. 원root삭제는하지않는다. 첫release GUI nativecompile/실행을 시작하며 결과는아직미확정이다.

원release/ACL계측 두실행은 앱 spawn 전 PreparedOwner의 ACL count에서 실패했다. actual SDDL은 self FA와 inherit-only OICI GA 두ACE였다. root는 원거절을보존하고 seed 신규owned디렉터리의 generic GA를concrete FA로바꿨다. 현재SID만 full-control이라는권한범위와protectedACL·단일ACE 관측기검사를유지했으며 기존fixtureACL은변경하지않았다. fresh release-fa 재시험을시작했다.

Windows release-fa 및timeout-fa 실제privateGUI는모두통과했다. 각root-review.json에서source SHA·20개ACK전원processalive표본·원CAP/파일hash·자연exit0/Job자연0·desktopclose/input불변 및actualPNG를독립확인했다. timeout20028ms, release3661ms다. read-onlySQLite integrity ok,원old.mp4 DB1행/path/size/nativefileidentity동일,미등록new.mp4미공개이며두파일/sentinel원bytes보존이다. 원fixture는boundedarchive후보존했고삭제는하지않았다. 최종FA seed를포함한fixture 및기본CPU전체4명령을순차검증중이며완료는최종결과를따른다.

최종 FA seed를포함한fixture 및기본CPU의fmt/clippy/test/release 총8명령이모두통과했다. 각windows-scan-closewait-final-{fixture,default}-20261005-resume/root-review.json에서현재source·고정artifact와GUI두case동일featureexe를독립대조했다. 기본OFF제품 release를최종target에복원했고Cargo.lock은불변이다. 실제첫관측경계의release/timeout·rollback·ACK·자연종료라는이작업범위를완료했다. 모든filesystem/SQLite호출즉시취소·대형scanpeak·전체R03/P0완료로확대하지않는다. 원fixture namespace는archive후보존상태다.

고정최종exe크기는fixture7875584B,기본7858176B로fixture추가17408B이다. 기본OFF에서는원nativeevent초기화/대기API가빌드에포함되지않으며새dependency/lock변경없음이다. 활성fixture worker의20초대기는시험전용이며제품scan에추가하지않는다.
